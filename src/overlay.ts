import './app.css';
import OverlayApp from './routes/OverlayApp.svelte';
import { mount } from 'svelte';

const target = document.getElementById('overlay');
if (!target) throw new Error('#overlay missing');
mount(OverlayApp, { target });
