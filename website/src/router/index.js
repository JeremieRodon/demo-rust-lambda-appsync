import { createRouter, createWebHistory } from 'vue-router';
import { getCurrentUser, signInWithRedirect } from 'aws-amplify/auth';

import AdminView from '@/views/AdminView.vue';
import MainLayout from '@/layouts/MainLayout.vue';
import LeaderBoard from '@/views/LeaderBoard.vue';
import GameView from '@/views/GameView.vue';

const routes = [
  {
    path: '/',
    name: 'mainlayout',
    component: MainLayout,
    children: [
      { path: '/', redirect: { name: 'game' } },
      {
        path: '/game',
        name: 'game',
        component: GameView,
      },
      {
        path: '/leaderboard',
        name: 'leaderboard',
        component: LeaderBoard,
      },
      {
        path: '/admin',
        name: 'admin',
        component: AdminView,
        meta: {
          requiresAuth: true,
        },
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
  scrollBehavior(to) {
    if (to.hash) {
      return {
        el: to.hash,
      };
    }
    return { top: 0 };
  },
});
router.beforeResolve((to, from, next) => {
  if (to.meta.requiresAuth) {
    getCurrentUser()
      .then(() => {
        next();
      })
      .catch(() => {
        window.sessionStorage.setItem('preauthpath', to.path);
        signInWithRedirect();
      });
  } else if (window.sessionStorage.getItem('preauthpath') != null) {
    getCurrentUser()
      .then(() => {
        const path = window.sessionStorage.getItem('preauthpath');
        window.sessionStorage.removeItem('preauthpath');
        next({
          path,
        });
      })
      .catch(() => {
        next();
      });
  } else {
    next();
  }
});

export default router;
