const gameList = document.getElementById("game-list");

function joinGame(gameUuid) {
    const socketNamespace = io(`localhost:3000/game/${gameUuid}`);
    socketNamespace.emit("test");
}

/**
 * Creates a game node element.
 * 
 * @param {string} game - The name of the game.
 * @returns {HTMLElement} The created game node element.
 */
function createGameNode(game) {
    const gameNode = document.createElement("li");
    gameNode.className = "game";

    const gameLink = document.createElement("a");
    gameLink.textContent = game;
    gameLink.href = `/game.html?uuid=${game}`;

    gameNode.appendChild(gameLink);

    return gameNode;
}

function loadGameList() {
    socket.emit("list-games", null, ({data, error}) => {
        if (data) {
            data.forEach(game => gameList.appendChild(createGameNode(game)));
            console.debug({data});
        } else if (error) {
            console.error(error)
        }
    });
}