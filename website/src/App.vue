<script setup>
import { Hub } from 'aws-amplify/utils';
import { computed, onMounted, provide, ref, watch } from 'vue';
import { RouterView } from 'vue-router';
import router from '@/router';
import { getUserInfos } from './modules/utils';
import AlertDisplay from './components/AlertDisplay.vue';
import { generateClient } from 'aws-amplify/api';

const client = generateClient();
provide('appsync_client', client);
const admin_client = generateClient({
  authMode: 'userPool',
});
provide('appsync_admin_client', admin_client);

const signed_user = ref(null);
provide('signed_user', signed_user);
const signed_user_is_admin = computed(() => {
  return signed_user.value != null && signed_user.value.is_admin;
});
provide('signed_user_is_admin', signed_user_is_admin);

const registered_player_obj = ref(null);
provide('registered_player_obj', registered_player_obj);
watch(registered_player_obj, () => {
  if (registered_player_obj.value) {
    window.localStorage.setItem('user_obj', JSON.stringify(registered_player_obj.value));
  } else {
    window.localStorage.removeItem('user_obj');
  }
});

// By default, we go in darkmode
// that is: unless the user previously explicitly set "light"
const dark_mode = ref(window.localStorage.getItem('themepref') != 'light');
provide('dark_mode', dark_mode);
// When dark_mode changes, we store the appropriate user pref
watch(dark_mode, () => {
  // Save the theme choice
  window.localStorage.setItem('themepref', dark_mode.value ? 'dark' : 'light');
});

Hub.listen('auth', ({ payload }) => {
  switch (payload.event) {
    case 'signedIn':
      console.log('user have been signedIn successfully.');
      verify_user_signed_in();
      break;
    case 'signedOut':
      console.log('user have been signedOut successfully.');
      verify_user_signed_in();
      router.push(router.resolve('/'));
      break;
    case 'tokenRefresh':
      console.log('auth tokens have been refreshed.');
      verify_user_signed_in();
      break;
    case 'tokenRefresh_failure':
      console.log('failure while refreshing auth tokens.');
      break;
    case 'signInWithRedirect':
      console.log('signInWithRedirect API has successfully been resolved.');
      break;
    case 'signInWithRedirect_failure':
      console.log('failure while trying to resolve signInWithRedirect API.');
      break;
    case 'customOAuthState':
      console.log('custom state returned from CognitoHosted UI');
      break;
  }
});

async function verify_user_signed_in() {
  try {
    signed_user.value = await getUserInfos();
    console.log(signed_user.value);
    return true;
  } catch {
    signed_user.value = null;
    return false;
  }
}

async function verify_registration() {
  registered_player_obj.value = JSON.parse(window.localStorage.getItem('user_obj'));
}

onMounted(async () => {
  await verify_user_signed_in();
  verify_registration();
});
</script>

<template>
  <alert-display></alert-display>
  <router-view :class="dark_mode ? 'dark' : ''" />
</template>
