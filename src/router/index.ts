import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import ConfigView from '../views/ConfigView.vue'
import AboutView from '../views/AboutView.vue'
import LogView from '../views/LogView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/config',
      name: 'config',
      component: ConfigView
    },
    {
      path: '/about',
      name: 'about',
      component: AboutView
    },
    {
      path: '/logs',
      name: 'logs',
      component: LogView
    }
  ]
})

export default router
