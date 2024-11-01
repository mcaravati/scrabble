const urlParams = new URLSearchParams(window.location.search);
const GAME_UUID = urlParams.get('uuid');

let CONNECTED = false;
const socket = io(`localhost:3000/game/${GAME_UUID}`);

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

function fetchPlayerList() {
    socket.emit("player-list", null, ({data, error}) => {
        console.info(data)
        if (data) {
            playerListHandler(data);
        } else if (error) {
            console.error(error);
        }
    })
}

socket.on('connect', () => {
    console.info('Connected to the server!');
    CONNECTED = true;
    whoami();
    fetchPlayerList();

    socket.on("player-registered", ({data, error}) => {
        if (data) {
            clearGameList();
            playerListHandler(data);
        } else if (error) {
            console.error(error);
        }
    });
});