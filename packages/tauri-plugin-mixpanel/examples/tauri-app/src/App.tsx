import { useState } from "react";
import mixpanel from "tauri-plugin-mixpanel-api";

function App() {
  const [distinctId, setDistinctId] = useState<string | null>(null);

  async function trackEvent() {
    await mixpanel.track("Button Clicked", { source: "Tauri App" });
    console.log("Tracked event 'Button Clicked'");
  }

  async function identifyUser() {
    await mixpanel.identify("user_mixpanel_test");
    console.log("Identified user with ID: user_mixpanel_test");
    await getDistinctId();
  }

  async function peopleSet() {
    await mixpanel.people.set({ "$name": "Test User", "plan": "Premium" });
    console.log("Called people.set");
  }

  async function peopleSetOnce() {
    await mixpanel.people.set_once({ "First Seen": new Date().toISOString() });
    console.log("Called people.set_once");
  }

  async function registerProps() {
    await mixpanel.register({ "App Version": "1.0.0" });
    console.log("Called register");
  }

  async function registerOnceProps() {
    await mixpanel.register_once({ "Initial Source": "Organic" });
    console.log("Called register_once");
  }

  async function getDistinctId() {
    const id = await mixpanel.get_distinct_id();
    setDistinctId(id);
    console.log(`Got distinct id: ${id}`);
  }

  async function resetMixpanel() {
    await mixpanel.reset();
    console.log("Called reset");
    await getDistinctId();
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>
      <p>Current Distinct ID: {distinctId ?? 'Not set'}</p>

      <div className="row">
        <button onClick={trackEvent}>Track Event</button>
        <button onClick={identifyUser}>Identify User</button>
        <button onClick={peopleSet}>People Set</button>
        <button onClick={peopleSetOnce}>People Set Once</button>
        <button onClick={registerProps}>Register Props</button>
        <button onClick={registerOnceProps}>Register Props Once</button>
        <button onClick={getDistinctId}>Get Distinct ID</button>
        <button onClick={resetMixpanel}>Reset Mixpanel</button>
      </div>
    </div>
  );
}

export default App;
