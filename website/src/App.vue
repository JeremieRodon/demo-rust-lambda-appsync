<script setup>
import { Hub } from 'aws-amplify/utils';
import { onMounted, provide, ref, watch } from 'vue';
import { RouterView } from 'vue-router';
import router from '@/router';
import { getUserInfos } from './modules/utils';
import AlertDisplay from './components/AlertDisplay.vue';

const signed_admin = ref(null);
provide('signed_admin', signed_admin);

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
      verify_admin_signed_in();
      break;
    case 'signedOut':
      console.log('user have been signedOut successfully.');
      verify_admin_signed_in();
      router.push(router.resolve('/'));
      break;
    case 'tokenRefresh':
      console.log('auth tokens have been refreshed.');
      verify_admin_signed_in();
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

async function verify_admin_signed_in() {
  try {
    signed_admin.value = await getUserInfos();
    console.log(signed_admin.value);
    return true;
  } catch {
    signed_admin.value = null;
    return false;
  }
}

onMounted(async () => {
  await verify_admin_signed_in();
});
</script>

<template>
  <alert-display></alert-display>
  <router-view :class="dark_mode ? 'dark' : ''" />
</template>
