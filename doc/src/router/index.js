import Vue from 'vue';
import VueRouter from 'vue-router';
import Base from '../views/Base.vue';
import Home from '../views/Home.vue';
import Docs from '../views/Docs.vue';
import CSS from '../views/CSS.vue';
import Layout from '../views/Layout.vue';
import Reference from '../views/Reference.vue';
import ReferenceHome from '../views/ReferenceHome.vue';
import ReferencePage from '../views/ReferencePage.vue';
import ReferenceOverview from '../views/ReferenceOverview.vue';

Vue.use(VueRouter);

const routes = [
  {
    path: '/',
    name: 'Base',
    component: Base,
    children: [
      {
        path: '',
        name: 'Home',
        component: Home,
        props: true,
      },
      {
        path: 'docs',
        name: 'Docs',
        component: Docs,
        props: true,
      },
      {
        path: 'docs/layout',
        name: 'Layout',
        component: Layout,
        props: true,
      },
      {
        path: 'docs/css',
        name: 'CSS',
        component: CSS,
        props: true,
      },
      {
        path: 'docs/low-level',
        name: 'Reference',
        component: Reference,
        props: true,
        children: [
          {
            path: '',
            name: 'ReferenceHome',
            component: ReferenceHome,
            props: true,
          },
          {
            path: ':lang',
            name: 'ReferenceOverview',
            component: ReferenceOverview,
            props: true,
          },
          {
            path: ':lang/:mod/:item',
            name: 'ReferencePage',
            component: ReferencePage,
            props: true,
          },
        ],
      },
    ],
  },
  // {
  //   path: '/about',
  //   name: 'About',
  //   // route level code-splitting
  //   // this generates a separate chunk (about.[hash].js) for this route
  //   // which is lazy-loaded when the route is visited.
  //   component: () => import(/* webpackChunkName: "about" */ '../views/About.vue'),
  // },
];

const router = new VueRouter({
  routes,
});

export default router;
