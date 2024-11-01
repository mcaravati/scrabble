// Elements used for user registration
const usernameField = document.getElementById('username-field');
const registerButton = document.getElementById('register-button');
const clearButton = document.getElementById('clear-button');
const playerList = document.getElementById('player-list');

clearButton.addEventListener('click', () => {
  localStorage.clear("player_uuid");
});

function clearGameList() {
  while (playerList.firstChild) {
    playerList.removeChild(playerList.firstChild);
  }
}


function createPlayerNode(player) {
  const gameNode = document.createElement("li");
  gameNode.className = "player";
  gameNode.textContent = player.name;

  return gameNode;
}

function playerListHandler(list) {
  list.forEach(player => playerList.appendChild(createPlayerNode(player)));
}

registerButton.addEventListener('click', async () => {
  const username = usernameField.value.trim();

  // TODO: Implement user notification on error
  if (username) {
    console.log(`Registering ${username}`)
    socket.emit("register_request", {
      game_uuid: GAME_UUID,
      username
    }, ({data, error}) => { // TODO: Implement error handling
      if (data) {
        localStorage.setItem("player_uuid", data);
        console.info(`User ${username} successfully registered with UUID ${data}`);
      } else if (error) {
        console.error(error);
      }
    });
  }
});