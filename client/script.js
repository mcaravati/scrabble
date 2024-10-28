const socket = io('localhost:3000');

socket.on('connect', () => {
    console.info('Connected to the server!');
});

document.addEventListener('DOMContentLoaded', () => { });