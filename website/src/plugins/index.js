/**
 * plugins/index.js
 *
 * Automatically included in `./src/main.js`
 */

// Plugins
import { loadAmplify } from './amplify';
import router from '../router';

export function registerPlugins(app) {
  loadAmplify();
  app.use(router);
}
