import './app.css';
import { mount } from 'svelte';
import MainApp from './routes/MainApp.svelte';
import FaceToFaceView from './routes/FaceToFaceView.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('#app missing');

// ponytail: userAgent sniff beats adding @tauri-apps/plugin-os just to read the platform.
const isAndroid = /Android/i.test(navigator.userAgent);
mount(isAndroid ? FaceToFaceView : MainApp, { target });
