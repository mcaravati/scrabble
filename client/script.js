const socket = io('localhost:3000');

socket.on('connect', () => {
    console.log('Connected to the server!');
    socket.emit('auth', { foo: 'bar' });
});


document.addEventListener('DOMContentLoaded', () => {
    const registerButton = document.getElementById("register-button");
    const logoutButton = document.getElementById("logout-button");
    const testButton = document.getElementById("send-message-button");

    // Check if user just refreshed the page
    const uuid = localStorage.getItem("UUID");
    if (uuid) {
        socket.emit("login-request", { uuid }, (data) => {
            console.log({data});
        });
    }

    testButton.addEventListener('click', () => {
        // socket.emit('message', { foo: 'bar' });
        socket.emit('register_request', { username: 'username_test' }, (data) => {
            console.log(`DEBUG:`);
            console.log(data)
        });

    });

    // sendMessageWithAckButton.addEventListener('click', () => {
    //     socket.emit('message-with-ack', { foo: 'bar' }, (data) => {
    //         console.log(`Received acknowledgement: ${data}`);
    //     });
    // });

    logoutButton.addEventListener('click', () => {
        socket.emit('logout', (data) => {
            console.log(`Received: ${data}`);
        });
    });
});