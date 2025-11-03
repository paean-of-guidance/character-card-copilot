import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('../views/Home.vue')
    },
    {
      path: '/editor',
      name: 'editor',
      component: () => import('../views/Editor.vue')
    },
    {
      path: '/editor/:uuid',
      name: 'editor-with-uuid',
      component: () => import('../views/Editor.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/Settings.vue')
    }
  ]
});

export default router;
