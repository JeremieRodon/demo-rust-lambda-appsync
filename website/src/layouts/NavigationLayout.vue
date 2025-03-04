<script setup>
import { RouterView } from 'vue-router';
import ThemeSwitch from '@/components/ThemeSwitch.vue';
import NavbarLink from '@/components/NavbarLink.vue';
import NavigationComponent from '@/components/NavigationComponent.vue';
import LoginButton from '@/components/LoginButton.vue';
import { computed, inject } from 'vue';

const signed_user = inject('signed_user');
const user_is_admin = computed(() => {
  if (signed_user.value) {
    return signed_user.value.is_admin;
  }
  return false;
});
</script>

<template>
  <navigation-component>
    <template v-slot:nav-title>
      <div class="flex flex-row flex-nowrap">
        <div class="w-16">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="fill-red-500">
            <path
              d="M12,21.35L10.55,20.03C5.4,15.36 2,12.27 2,8.5C2,5.41 4.42,3 7.5,3C9.24,3 10.91,3.81 12,5.08C13.09,3.81 14.76,3 16.5,3C19.58,3 22,5.41 22,8.5C22,12.27 18.6,15.36 13.45,20.03L12,21.35Z"
            />
          </svg>
        </div>
        <div class="content-center text-center font-bold uppercase">
          <div>Le PACS de</div>
          <div>Jerem & Tine</div>
        </div>
      </div>
    </template>
    <template v-slot:nav-center>
      <navbar-link to="/home">
        <span class="hidden md:flex">Accueil</span>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-8 w-8 fill-current flex md:hidden"
          viewBox="0 0 24 24"
        >
          <path d="M10,20V14H14V20H19V12H22L12,3L2,12H5V20H10Z" />
        </svg>
      </navbar-link>
      <navbar-link v-if="user_is_admin" to="/admin">
        <span class="hidden md:flex">L'administration</span>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-8 w-8 fill-current flex md:hidden"
          viewBox="0 0 24 24"
        >
          <path
            d="M12 1L21 5V11C21 16.55 17.16 21.74 12 23C6.84 21.74 3 16.55 3 11V5L12 1M16 14H8V15.5C8 15.77 8.19 15.96 8.47 16L8.57 16H15.43C15.74 16 15.95 15.84 16 15.59L16 15.5V14M17 8L17 8L14.33 10.67L12 8.34L9.67 10.67L7 8L7 8L8 13H16L17 8Z"
          />
        </svg>
      </navbar-link>
      <navbar-link v-if="!user_is_admin" to="/register">
        <span class="hidden md:flex">L'inscription</span>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-8 w-8 fill-current flex md:hidden"
          viewBox="0 0 24 24"
        >
          <path
            d="M15 5H18L16.5 7L15 5M5 2H19C20.11 2 21 2.9 21 4V20C21 21.11 20.11 22 19 22H5C3.9 22 3 21.11 3 20V4C3 2.9 3.9 2 5 2M5 4V8H19V4H5M5 20H19V10H5V20M7 12H17V14H7V12M7 16H17V18H7V16Z"
          />
        </svg>
      </navbar-link>
      <navbar-link to="/photos">
        <span class="hidden md:flex">Les Photos</span>

        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-8 w-8 fill-current flex md:hidden"
          viewBox="0 0 24 24"
        >
          <path
            d="M4,4H7L9,2H15L17,4H20A2,2 0 0,1 22,6V18A2,2 0 0,1 20,20H4A2,2 0 0,1 2,18V6A2,2 0 0,1 4,4M12,7A5,5 0 0,0 7,12A5,5 0 0,0 12,17A5,5 0 0,0 17,12A5,5 0 0,0 12,7M12,9A3,3 0 0,1 15,12A3,3 0 0,1 12,15A3,3 0 0,1 9,12A3,3 0 0,1 12,9Z"
          />
        </svg>
      </navbar-link>
      <navbar-link to="/game">
        <span class="hidden md:flex">Les Jeux</span>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-8 w-8 fill-current flex md:hidden"
          viewBox="0 0 24 24"
        >
          <path
            d="M7.97,16L5,19C4.67,19.3 4.23,19.5 3.75,19.5A1.75,1.75 0 0,1 2,17.75V17.5L3,10.12C3.21,7.81 5.14,6 7.5,6H16.5C18.86,6 20.79,7.81 21,10.12L22,17.5V17.75A1.75,1.75 0 0,1 20.25,19.5C19.77,19.5 19.33,19.3 19,19L16.03,16H7.97M7,8V10H5V11H7V13H8V11H10V10H8V8H7M16.5,8A0.75,0.75 0 0,0 15.75,8.75A0.75,0.75 0 0,0 16.5,9.5A0.75,0.75 0 0,0 17.25,8.75A0.75,0.75 0 0,0 16.5,8M14.75,9.75A0.75,0.75 0 0,0 14,10.5A0.75,0.75 0 0,0 14.75,11.25A0.75,0.75 0 0,0 15.5,10.5A0.75,0.75 0 0,0 14.75,9.75M18.25,9.75A0.75,0.75 0 0,0 17.5,10.5A0.75,0.75 0 0,0 18.25,11.25A0.75,0.75 0 0,0 19,10.5A0.75,0.75 0 0,0 18.25,9.75M16.5,11.5A0.75,0.75 0 0,0 15.75,12.25A0.75,0.75 0 0,0 16.5,13A0.75,0.75 0 0,0 17.25,12.25A0.75,0.75 0 0,0 16.5,11.5Z"
          />
        </svg>
      </navbar-link>
    </template>
    <template v-slot:nav-end>
      <theme-switch />
      <login-button />
    </template>
    <router-view />
  </navigation-component>
</template>
