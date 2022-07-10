function wait(ms){
   var start = new Date().getTime();
   var end = start;
   while(end < start + ms) {
     end = new Date().getTime();
  }
}

function rainbow() {
  document.getElementById('name_text').style.color = "red";
  setTimeout(function(){
    document.getElementById('name_text').style.color = "orange";
  }, 200);
  setTimeout(function(){
    document.getElementById('name_text').style.color = "yellow";
  }, 400);
  setTimeout(function(){
    document.getElementById('name_text').style.color = "green";
  }, 600);
  setTimeout(function(){
    document.getElementById('name_text').style.color = "blue";
  }, 800);
  setTimeout(function(){
    document.getElementById('name_text').style.color = "purple";
  }, 1000);
  setTimeout(function(){
    document.getElementById('name_text').style.color = "DeepPink";
  }, 1200);
}
