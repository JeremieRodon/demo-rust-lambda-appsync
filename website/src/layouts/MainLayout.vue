<script setup>
import { RouterView } from 'vue-router';
import ThemeSwitch from '@/components/ThemeSwitch.vue';
import { ref } from 'vue';

const drawer_open = ref(false);
</script>

<template>
  <div class="drawer min-h-screen">
    <input type="checkbox" id="nav-drawer" class="drawer-toggle" v-model="drawer_open" />
    <div class="drawer-content flex flex-col">
      <header class="h-16">
        <!-- Navbar -->
        <nav
          class="fixed top-0 left-0 z-10 bg-base-100/90 navbar py-0 shadow-sm shadow-base-content/50 w-full h-16"
        >
          <div class="navbar-start hidden md:flex">
            <button class="btn btn-square btn-ghost" @click="drawer_open = !drawer_open">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                class="inline-block h-5 w-5 stroke-current"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 6h16M4 12h16M4 18h16"
                ></path>
              </svg>
            </button>
          </div>
          <div class="navbar-center">
            <img src="@/assets/logo.webp" class="h-14" />
            <div class="text-2xl font-black ml-2">
              <div>Benchmark</div>
              <div>Game</div>
            </div>
          </div>
          <div class="navbar-end">
            <theme-switch></theme-switch>
            <div class="basis-4"></div>
          </div>
        </nav>
      </header>
      <!-- Page content here -->
      <router-view />
      <!-- Footer -->
      <div class="grow"></div>
      <footer class="footer justify-center md:justify-end pb-20 pt-4 md:py-4 md:p-4">
        <aside class="text-xs">
          <a
            href="https://github.com/JeremieRodon/demo-rust-lambda-appsync.git"
            class="p-1"
            target="_blank"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              class="h-6 inline fill-current"
            >
              <path
                d="M12,2A10,10 0 0,0 2,12C2,16.42 4.87,20.17 8.84,21.5C9.34,21.58 9.5,21.27 9.5,21C9.5,20.77 9.5,20.14 9.5,19.31C6.73,19.91 6.14,17.97 6.14,17.97C5.68,16.81 5.03,16.5 5.03,16.5C4.12,15.88 5.1,15.9 5.1,15.9C6.1,15.97 6.63,16.93 6.63,16.93C7.5,18.45 8.97,18 9.54,17.76C9.63,17.11 9.89,16.67 10.17,16.42C7.95,16.17 5.62,15.31 5.62,11.5C5.62,10.39 6,9.5 6.65,8.79C6.55,8.54 6.2,7.5 6.75,6.15C6.75,6.15 7.59,5.88 9.5,7.17C10.29,6.95 11.15,6.84 12,6.84C12.85,6.84 13.71,6.95 14.5,7.17C16.41,5.88 17.25,6.15 17.25,6.15C17.8,7.5 17.45,8.54 17.35,8.79C18,9.5 18.38,10.39 18.38,11.5C18.38,15.32 16.04,16.16 13.81,16.41C14.17,16.72 14.5,17.33 14.5,18.26C14.5,19.6 14.5,20.68 14.5,21C14.5,21.27 14.66,21.59 15.17,21.5C19.14,20.16 22,16.42 22,12A10,10 0 0,0 12,2Z"
              />
            </svg>
            Deploy your own!
          </a>
        </aside>
      </footer>
      <div class="dock bg-base-100 flex-none md:hidden h-16 z-20">
        <router-link to="/game">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-8 w-8 fill-current flex md:hidden"
            viewBox="0 0 24 24"
          >
            <path
              d="M7.97,16L5,19C4.67,19.3 4.23,19.5 3.75,19.5A1.75,1.75 0 0,1 2,17.75V17.5L3,10.12C3.21,7.81 5.14,6 7.5,6H16.5C18.86,6 20.79,7.81 21,10.12L22,17.5V17.75A1.75,1.75 0 0,1 20.25,19.5C19.77,19.5 19.33,19.3 19,19L16.03,16H7.97M7,8V10H5V11H7V13H8V11H10V10H8V8H7M16.5,8A0.75,0.75 0 0,0 15.75,8.75A0.75,0.75 0 0,0 16.5,9.5A0.75,0.75 0 0,0 17.25,8.75A0.75,0.75 0 0,0 16.5,8M14.75,9.75A0.75,0.75 0 0,0 14,10.5A0.75,0.75 0 0,0 14.75,11.25A0.75,0.75 0 0,0 15.5,10.5A0.75,0.75 0 0,0 14.75,9.75M18.25,9.75A0.75,0.75 0 0,0 17.5,10.5A0.75,0.75 0 0,0 18.25,11.25A0.75,0.75 0 0,0 19,10.5A0.75,0.75 0 0,0 18.25,9.75M16.5,11.5A0.75,0.75 0 0,0 15.75,12.25A0.75,0.75 0 0,0 16.5,13A0.75,0.75 0 0,0 17.25,12.25A0.75,0.75 0 0,0 16.5,11.5Z"
            />
          </svg>
          <div class="font-light text-xs">Game</div>
        </router-link>
        <router-link to="/leaderboard">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-8 w-8 fill-current flex md:hidden"
            viewBox="0 0 24 24"
          >
            <path
              d="M18 2C17.1 2 16 3 16 4H8C8 3 6.9 2 6 2H2V11C2 12 3 13 4 13H6.2C6.6 15 7.9 16.7 11 17V19.08C8 19.54 8 22 8 22H16C16 22 16 19.54 13 19.08V17C16.1 16.7 17.4 15 17.8 13H20C21 13 22 12 22 11V2H18M6 11H4V4H6V11M20 11H18V4H20V11Z"
            />
          </svg>
          <div class="font-light text-xs">Leaderboard</div>
        </router-link>
      </div>
    </div>
    <div class="drawer-side z-20 hidden md:grid">
      <label for="nav-drawer" aria-label="close sidebar" class="drawer-overlay"></label>
      <nav class="flex flex-col justify-between bg-base-200 min-h-full w-fit max-w-[60%] p-4">
        <div class="grow"></div>
        <div class="flex flex-col" @click="drawer_open = false">
          <router-link
            to="/"
            class="h-full px-2 xl:px-4 content-center font-bold uppercase hover:bg-primary/10"
            activeClass="active"
            tabindex="-1"
          >
            Game
          </router-link>
          <router-link
            to="/leaderboard"
            class="h-full px-2 xl:px-4 content-center font-bold uppercase hover:bg-primary/10"
            activeClass="active"
            tabindex="-1"
          >
            Leaderboard
          </router-link>
        </div>
        <div class="grow"></div>
      </nav>
    </div>
  </div>
</template>
