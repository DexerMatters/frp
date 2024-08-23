window.onload = function () {
  // Register mouse events
  ['mousemove', 'mouseup', 'mousedown'].forEach(function (eventName) {
    window.addEventListener(eventName, function (e) {
      console.log('mousedown');
      fetch('/mouse-event', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          x: e.clientX,
          y: e.clientY,
          button: e.button,
          name: eventName
        })
      });
    });
  });
}
