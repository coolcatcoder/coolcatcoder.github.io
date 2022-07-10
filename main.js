function wait(ms){
   var start = new Date().getTime();
   var end = start;
   while(end < start + ms) {
     end = new Date().getTime();
  }
}

function rainbow() {
  document.getElementById('main_title').style.color = "red";
  setTimeout(function(){
    document.getElementById('main_title').style.color = "orange";
  }, 200);
  setTimeout(function(){
    document.getElementById('main_title').style.color = "yellow";
  }, 400);
  setTimeout(function(){
    document.getElementById('main_title').style.color = "green";
  }, 600);
  setTimeout(function(){
    document.getElementById('main_title').style.color = "blue";
  }, 800);
  setTimeout(function(){
    document.getElementById('main_title').style.color = "purple";
  }, 1000);
  setTimeout(function(){
    document.getElementById('main_title').style.color = "pink";
  }, 1200);
}
