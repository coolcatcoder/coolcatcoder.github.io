function wait(ms){
   var start = new Date().getTime();
   var end = start;
   while(end < start + ms) {
     end = new Date().getTime();
  }
}

function evil() {
  var notification_list = [
    "Don't forget to check our site for new articles!",
    "Click here to learn to code CSS!",
    "Click here to learn to code JavaScript!",
    "Click here to learn to code HTML!"
  ];
  var i = 0
  while (i < 30) {
    var notification = new Notification(notification_list[Math.floor(Math.random() * 4)]);
    wait(10000);
    i += 1
  }
}

function annoy() {
  document.getElementById('awful').innerHTML = "<p id=\"annoying\">To view this article please click \"allow\" to confirm you are not a robot.</p>";
  Notification.requestPermission().then(function (permission) {
    if (permission === "granted") {
      var notification = new Notification("Thank you for subscribing to our newsletter!");
      evil();
      document.getElementById('awful').innerHTML = "<p id=\"annoying\">Thank you for confirming that you are not a robot.<br>You will be redirected to the article soon.</p>";
    } else {
      document.getElementById('awful').innerHTML = "<p id=\"annoying\">You did not press allow, as such we cannot confirm that you are not a robot.</p>";
    }
  });
}
