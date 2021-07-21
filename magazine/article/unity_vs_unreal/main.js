function annoy() {
  getElementById('awful').innerHTML = "<p id=\"annoying\">To view this article please click \"allow\" to confirm you are not a robot.</p>";
  Notification.requestPermission().then(funtion (permission) {
    if (permission === "granted") {
      var notification = new Notification("Thank you for subscribing to our newsletter!");
      getElementById('awful').innerHTML = "<p id=\"annoying\">Thank you for confirming that you are not a robot</p>";
    } else {
      getElementById('awful').innerHTML = "<p id=\"annoying\">You did not press allow, as such we cannot confirm that you are not a robot.</p>";
    }
  });
}
