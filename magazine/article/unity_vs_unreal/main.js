function annoy() {
  document.getElementById('awful').innerHTML = "<p id=\"annoying\">To view this article please click \"allow\" to confirm you are not a robot.</p>";
  Notification.requestPermission().then(function (permission) {
    if (permission === "granted") {
      var notification = new Notification("Thank you for subscribing to our newsletter!");
      document.getElementById('awful').innerHTML = "<p id=\"annoying\">Thank you for confirming that you are not a robot</p>";
    } else {
      document.getElementById('awful').innerHTML = "<p id=\"annoying\">You did not press allow, as such we cannot confirm that you are not a robot.</p>";
    }
  });
}
