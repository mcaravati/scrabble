const gameList = document.getElementById("game-list");

function loadGameList() {
    socket.emit("list-games", null, ({data, error}) => {
        if (data) {
            data.forEach(game => {
                const gameElement = document.createElement("div");

                gameElement.textContent = game;
                gameElement.className = "game";

                gameList.appendChild(gameElement);
            })
            console.debug({data});
        } else if (error) {
            console.error(error)
        }
    });
}