/* @refresh reload */
import { render } from 'solid-js/web';
import { MetaProvider } from '@solidjs/meta';
import App from './App';
import './styles.css';

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error('Root element not found.');
}

render(
  () => (
    <MetaProvider>
      <App />
    </MetaProvider>
  ),
  root!,
);
