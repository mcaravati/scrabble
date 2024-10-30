let CONNECTED = false;
const socket = io('localhost:3000');

function whoami() {
    let player_uuid = localStorage.getItem("player_uuid");

    if (player_uuid) {
        socket.emit("whoami", {
            player_uuid
        }, ({data, error}) => {
            if (data) {
                console.log({data});
            } else {
                localStorage.clear("player_uuid");
                console.error(error);
            }
        });
    }
  }

socket.on('connect', () => {
    console.info('Connected to the server!');
    CONNECTED = true;
    whoami();
    loadGameList();
});

document.addEventListener('DOMContentLoaded', () => {
    socket.emit("list-games", data => {
        console.debug({data});
    })
});