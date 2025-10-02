import { createFileRoute } from "@tanstack/react-router";
import useWebSocket from "@/hooks/use-websocket";

export const Route = createFileRoute("/")({
  component: App,
});

function App() {
  const websocket = useWebSocket("ws://localhost:4000");
  return <div>{websocket.isConnected ? "Connected" : "disconnected"}</div>;
}
