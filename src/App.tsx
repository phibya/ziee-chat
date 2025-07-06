import {useState} from "react";
import reactLogo from "./assets/react.svg";
import "./App.css";
import {ApiClient} from "./api/client.ts";

function App() {
    const [greetMsg, setGreetMsg] = useState("");
    const [name, setName] = useState("");

    async function greet() {
        // Clean, type-safe API call using the ApiClient
        ApiClient.User.greet({name}).then((response) => {
            console.log("Response from API:", response);
            setGreetMsg(response); // response is typed as string
        })
    }

    return (
        <main className="container">
            <h1>Welcome to Tauri + React</h1>

            <div className="row">
                <a href="https://vitejs.dev" target="_blank">
                    <img src="/vite.svg" className="logo vite" alt="Vite logo"/>
                </a>
                <a href="https://tauri.app" target="_blank">
                    <img src="/tauri.svg" className="logo tauri" alt="Tauri logo"/>
                </a>
                <a href="https://reactjs.org" target="_blank">
                    <img src={reactLogo} className="logo react" alt="React logo"/>
                </a>
            </div>
            <p>Click on the Tauri , Vite, and React logos to learn more.</p>

            <form
                className="row"
                onSubmit={(e) => {
                    e.preventDefault();
                    greet().catch(console.error);
                }}
            >
                <input
                    id="greet-input"
                    onChange={(e) => setName(e.currentTarget.value)}
                    placeholder="Enter a name..."
                />
                <button type="submit">Greet</button>
            </form>
            <p>{greetMsg}</p>
        </main>
    );
}

export default App;
