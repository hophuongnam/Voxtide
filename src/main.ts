import './app.css';
import MainApp from './routes/MainApp.svelte';
import { mount } from 'svelte';

const target = document.getElementById('app');
if (!target) throw new Error('#app missing');
mount(MainApp, { target });
