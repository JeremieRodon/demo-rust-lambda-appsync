mod graphql;

use graphql::GraphQLSchema;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{
    Ident as SynIdent, LitBool, LitStr, Token, Type, parenthesized, parse::Parse, parse_macro_input,
};

struct AWSClient {
    fct_identifier: Ident,
    client_type: Type,
}
impl Parse for AWSClient {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Expected example:
        // dynamodb() -> aws_sdk_dynamodb::Client
        let fct_identifier = input.parse::<Ident>()?;
        let _empty;
        _ = parenthesized!(_empty in input);
        _ = input.parse::<Token![->]>()?;
        let client_type = input.parse::<syn::Type>()?;
        Ok(Self {
            fct_identifier,
            client_type,
        })
    }
}
impl AWSClient {
    fn is_next(input: syn::parse::ParseStream) -> bool {
        Self::parse(&input.fork()).is_ok()
    }
    fn aws_config_getter() -> TokenStream2 {
        quote! {
            static AWS_SDK_CONFIG: ::std::sync::OnceLock<::lambda_appsync::aws_config::SdkConfig> = ::std::sync::OnceLock::new();
            pub fn aws_sdk_config() -> &'static ::lambda_appsync::aws_config::SdkConfig {
                AWS_SDK_CONFIG.get().unwrap()
            }
        }
    }
    fn aws_config_init() -> TokenStream2 {
        quote! {
            AWS_SDK_CONFIG.set(::lambda_appsync::aws_config::load_from_env().await).unwrap();
        }
    }
    fn aws_client_getter(&self) -> impl ToTokens {
        let Self {
            fct_identifier,
            client_type,
        } = self;
        quote! {
            pub fn #fct_identifier() -> #client_type {
                static CLIENT: ::std::sync::OnceLock<#client_type> = ::std::sync::OnceLock::new();
                CLIENT.get_or_init(||<#client_type>::new(aws_sdk_config())).clone()
            }
        }
    }
}
impl ToTokens for AWSClient {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            fct_identifier,
            client_type,
        } = self;
        let fct_identifier = LitStr::new(&fct_identifier.to_string(), fct_identifier.span());
        let client_type = LitStr::new(&format!("{client_type:?}"), fct_identifier.span());

        tokens.extend(quote! {
            AWSClient {
                fct_identifier: #fct_identifier,
                client_type: #client_type,
            }
        });
    }
}

struct AppsyncLambdaMainParams {
    graphql_schema_path: LitStr,
    is_batch_integration: bool,
    aws_clients: Vec<AWSClient>,
    hook: Option<Ident>,
}

enum AppsyncLambdaMainOptionalParameter {
    Batch(bool),
    Hook(Ident),
}
impl Parse for AppsyncLambdaMainOptionalParameter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        _ = input.parse::<Token![=]>()?;
        match ident.to_string().as_str() {
            "batch" => Ok(Self::Batch(input.parse::<LitBool>()?.value())),
            "hook" => Ok(Self::Hook(input.parse()?)),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unknown parameter `{ident}`",),
            )),
        }
    }
}

impl Parse for AppsyncLambdaMainParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let graphql_schema_path = input.parse::<LitStr>()?;
        let mut is_batch_integration = true;
        let mut aws_clients = vec![];
        let mut hook = None;

        while input.peek(Token![,]) {
            _ = input.parse::<Token![,]>()?;
            if input.peek(SynIdent) && input.peek2(Token![=]) {
                // That's a parameter
                match input.parse::<AppsyncLambdaMainOptionalParameter>()? {
                    AppsyncLambdaMainOptionalParameter::Batch(is_batch) => {
                        is_batch_integration = is_batch
                    }
                    AppsyncLambdaMainOptionalParameter::Hook(ident) => hook = Some(ident),
                }
            } else if AWSClient::is_next(input) {
                aws_clients.push(input.parse::<AWSClient>()?);
            } else {
                return Err(syn::Error::new(input.span(), format!("Unknown argument",)));
            }
        }

        Ok(Self {
            graphql_schema_path,
            is_batch_integration,
            aws_clients,
            hook,
        })
    }
}

struct AppsyncLambdaMain {
    graphql_schema: GraphQLSchema,
    with_batch: bool,
    aws_clients: Vec<AWSClient>,
    hook: Option<Ident>,
}
impl AppsyncLambdaMain {
    fn appsync_event_handler(&self, tokens: &mut TokenStream2) {
        let call_hook = if let Some(ref hook) = self.hook {
            quote_spanned! {hook.span()=>
                if let Some(resp) = #hook(&event).await {
                    return resp;
                }
            }
        } else {
            quote! {}
        };
        tokens.extend(quote! {
            async fn appsync_handler(event: ::lambda_appsync::AppSyncEvent<Operation>) -> ::lambda_appsync::AppSyncResponse {
                ::lambda_appsync::log::info!("event={event:?}");
                ::lambda_appsync::log::info!("operation={:?}", event.info.operation);

                #call_hook
                let ::lambda_appsync::AppSyncEvent {
                    identity: _,
                    request: _,
                    source: _,
                    info:
                        ::lambda_appsync::AppSyncEventInfo {
                            operation,
                            selection_set_graphql: _,
                            selection_set_list: _,
                            variables: _,
                        },
                    args,
                } = event;
                ::lambda_appsync::log::info!("operation={operation:?}");
                operation.execute(args).await
            }
        });
        if self.with_batch {
            tokens.extend(quote! {
                async fn appsync_batch_handler(
                    events: Vec<::lambda_appsync::AppSyncEvent<Operation>>,
                ) -> Vec<::lambda_appsync::AppSyncResponse> {
                    let handles = events
                        .into_iter()
                        .map(|e| ::lambda_appsync::tokio::spawn(appsync_handler(e)))
                        .collect::<Vec<_>>();

                    let mut results = vec![];
                    for h in handles {
                        results.push(h.await.unwrap())
                    }
                    results
                }

            });
        }
    }

    fn lambda_main(&self, tokens: &mut TokenStream2) {
        let (config_init, config_getter) = if self.aws_clients.len() > 0 {
            (AWSClient::aws_config_init(), AWSClient::aws_config_getter())
        } else {
            (TokenStream2::new(), TokenStream2::new())
        };
        let aws_client_getters = self.aws_clients.iter().map(|ac| ac.aws_client_getter());

        let (appsync_handler, ret_type) = if self.with_batch {
            (
                format_ident!("appsync_batch_handler"),
                quote! {Vec<::lambda_appsync::AppSyncResponse>},
            )
        } else {
            (
                format_ident!("appsync_handler"),
                quote! {::lambda_appsync::AppSyncResponse},
            )
        };

        tokens.extend(quote! {
            async fn function_handler(
                event: ::lambda_appsync::lambda_runtime::LambdaEvent<::lambda_appsync::serde_json::Value>,
            ) -> Result<#ret_type, ::lambda_appsync::lambda_runtime::Error> {
                ::lambda_appsync::log::debug!("{event:?}");
                ::lambda_appsync::log::info!("{}", ::lambda_appsync::serde_json::json!(event.payload));
                Ok(#appsync_handler(::lambda_appsync::serde_json::from_value(event.payload)?).await)
            }

            #config_getter

            #(#aws_client_getters)*

            use ::lambda_appsync::tokio;
            #[tokio::main]
            async fn main() -> Result<(), ::lambda_appsync::lambda_runtime::Error> {
                ::lambda_appsync::env_logger::Builder::from_env(
                    ::lambda_appsync::env_logger::Env::default()
                        .default_filter_or("info,tracing::span=warn")
                        .default_write_style_or("never"),
                )
                .format_timestamp_micros()
                .init();

                #config_init

                ::lambda_appsync::lambda_runtime::run(::lambda_appsync::lambda_runtime::service_fn(function_handler)).await
            }
        });
    }
}

impl TryFrom<AppsyncLambdaMainParams> for AppsyncLambdaMain {
    type Error = syn::Error;

    fn try_from(params: AppsyncLambdaMainParams) -> Result<Self, Self::Error> {
        let AppsyncLambdaMainParams {
            graphql_schema_path,
            is_batch_integration,
            aws_clients,
            hook,
        } = params;

        let schema_str = std::fs::read_to_string(&graphql_schema_path.value()).map_err(|e| {
            syn::Error::new(
                graphql_schema_path.span(),
                format!("Could not open GraphQL schema file ({e})",),
            )
        })?;
        let schema = graphql_parser::parse_schema(&schema_str)
            .map_err(|e| {
                syn::Error::new(
                    graphql_schema_path.span(),
                    format!("Could not parse GraphQL schema file ({e})",),
                )
            })?
            .into_static();
        let graphql_schema = GraphQLSchema::try_from((schema, graphql_schema_path.span()))?;
        Ok(Self {
            graphql_schema,
            with_batch: is_batch_integration,
            aws_clients,
            hook,
        })
    }
}

impl ToTokens for AppsyncLambdaMain {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.graphql_schema.to_tokens(tokens);
        self.appsync_event_handler(tokens);
        self.lambda_main(tokens);
    }
}

pub(crate) fn appsync_lambda_main_impl(input: TokenStream) -> TokenStream {
    let almp = parse_macro_input!(input as AppsyncLambdaMainParams);
    let alm = match AppsyncLambdaMain::try_from(almp) {
        Ok(alm) => alm,
        Err(e) => return e.into_compile_error().into(),
    };
    quote! {
        #alm
    }
    .into()
}
