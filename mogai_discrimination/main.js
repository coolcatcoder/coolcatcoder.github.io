function rainbow(element_to_rainbow) {
  element_to_rainbow.style.color = "red";
  setTimeout(function(){
    element_to_rainbow.style.color = "orange";
  }, 200);
  setTimeout(function(){
    element_to_rainbow.style.color = "yellow";
  }, 400);
  setTimeout(function(){
    element_to_rainbow.style.color = "green";
  }, 600);
  setTimeout(function(){
    element_to_rainbow.style.color = "blue";
  }, 800);
  setTimeout(function(){
    element_to_rainbow.style.color = "purple";
  }, 1000);
  setTimeout(function(){
    element_to_rainbow.style.color = "DeepPink";
  }, 1200);
}
