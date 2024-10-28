// Elements used for user registration
const usernameField = document.getElementById('username-field');
const registerButton = document.getElementById('register-button');

registerButton.addEventListener('click', async () => {
  const username = usernameField.value.trim();

  // TODO: Implement user notification on error
  if (username) {
    console.log(`Registering ${username}`)
    socket.emit("register_request", {
      game_uuid: "550e8400-e29b-41d4-a716-446655440000",
      username
    }, (userUuid) => { // TODO: Implement error handling
      localStorage.setItem("userUuid", userUuid);
      console.info(`User ${username} successfully registered with UUID ${userUuid}`);
    });
  }
});