import React, { useEffect } from "react";
import mixpanel from "tauri-plugin-mixpanel-api";

function App() {
  useEffect(() => {
    console.log(mixpanel);
  }, []);

  async function trackButtonClick() {
    try {
      await mixpanel.track("button_click", {
        button: "Track Button",
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri + React!</h1>
      <p>Click the button to track an event with Mixpanel.</p>

      <div className="row">
        <button onClick={trackButtonClick}>Track Event</button>
      </div>
    </div>
  );
}

export default App;
