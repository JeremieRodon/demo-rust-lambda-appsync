import { createRouter, createWebHistory } from 'vue-router';
import { getCurrentUser, signIn } from 'aws-amplify/auth';
import MainView from '@/views/MainView.vue';
import AdminView from '@/views/AdminView.vue';

const routes = [
  {
    path: '/',
    name: 'main',
    component: MainView,
  },
  {
    path: '/admin',
    name: 'admin',
    component: AdminView,
    meta: {
      requiresAuth: true,
    },
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
        signIn();
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
