/**
 * plugins/index.js
 *
 * Automatically included in `./src/main.js`
 */
import { Amplify } from 'aws-amplify';

const authConfig = {
  Cognito: {
    region: import.meta.env.VITE_AWS_REGION,
    // User pool configuration
    userPoolId: import.meta.env.VITE_COGNITO_USER_POOL_ID,
    userPoolClientId: import.meta.env.VITE_COGNITO_USER_POOL_WEB_CLIENT_ID,
    // Mandatory sign in
    mandatorySignIn: true,
    loginWith: {
      // Hosted UI configuration
      oauth: {
        domain: import.meta.env.VITE_COGNITO_DOMAIN,
        scopes: ['openid', 'email', 'profile'],
        redirectSignIn: [import.meta.env.VITE_WEBSITE_URL],
        redirectSignOut: [import.meta.env.VITE_WEBSITE_URL],
        responseType: 'code',
      },
    },
  },
};

const apiConfig = {
  GraphQL: {
    endpoint: import.meta.env.VITE_GRAPHQLAPI_URL,
    region: import.meta.env.VITE_AWS_REGION,
    defaultAuthMode: 'apiKey',
    apiKey: import.meta.env.VITE_GRAPHQLAPI_KEY,
  },
};

export function loadAmplify() {
  Amplify.configure({
    Auth: authConfig,
    API: apiConfig,
  });
}
