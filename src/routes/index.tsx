import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";

export const Route = createFileRoute("/")({
  component: App,
});

function App() {
  const [connected, setIsConnected] = useState(false);
  const [userId, setUserId] = useState(null);
  const [roomInput, setRoomInput] = useState("");
  const [roomName, setRoomName] = useState("");
  const [room, setRoom] = useState<string | null>(null);
  const [messagesList, setMessagesList] = useState<
    { by: string; message: string }[] | null
  >(null);
  const [msg, setMsg] = useState<string>("");

  const ws = useRef<WebSocket | null>(null);
  useEffect(() => {
    if (connected && ws.current) {
      ws.current.send(JSON.stringify({ type: "info" }));
    }
  }, [connected, ws.current]);

  useEffect(() => {
    (() => {
      const websocket = new WebSocket("ws://127.0.0.1:4000");
      ws.current = websocket;
      if (ws.current) {
        ws.current.onopen = () => {
          setIsConnected(true);
        };
        ws.current.onclose = () => {
          setIsConnected(false);
          setRoom(null);
          setMessagesList(null);
          setUserId(null);
        };
        ws.current.onmessage = (event) => {
          console.log(event.data);
          const message = JSON.parse(event.data);
          switch (message.type) {
            case "room_created":
              setRoom(message.room_id);
              setMessagesList([]);
              break;
            case "room_joined":
              setRoom(message.room_id);
              ws.current?.send(
                JSON.stringify({
                  type: "list_messages",
                  room: message.room_id,
                }),
              );
              break;
            case "list_messages":
              setMessagesList(message.messages);
              break;
            case "info":
              setUserId(message.user_id);
              break;
            case "room_broadcast":
              console.log("message received : ", message);
              const newMsg = {
                by: message.by,
                message: message.message,
              };
              setMessagesList((prev) => (prev ? [...prev, newMsg] : [newMsg]));
              break;
          }
        };
      }
    })();
    return () => ws.current?.close();
  }, []);

  return (
    <div className="m-6">
      {connected ? "connected" : "disconnected"}
      <div>
        {room ? (
          <div className="p-4 h-screen w-full grid space-y-4">
            <div className="w-full flex items-center justify-between">
              <h1 className="text-2xl font-bold">Send Message</h1>
              <p>{room}</p>
            </div>
            {messagesList && messagesList?.length > 0 ? (
              <ScrollArea className="border-border border rounded-md p-2 min-h-[400px] max-h-[800px] flex flex-col space-y-4">
                {messagesList.map((msg, index) => (
                  <div
                    className={`bg-primary ${msg.by === userId && "place-self-end"} text-primary-foreground rounded-xl flex flex-col gap-1 p-2 m-2 max-w-fit`}
                    key={index}
                  >
                    <p className="text-xs">{msg.by}</p>
                    <p className="text-lg font-semibold">{msg.message}</p>
                  </div>
                ))}
              </ScrollArea>
            ) : (
              <div className="border-border border rounded-md h-[400px] flex items-center justify-center">
                <h1 className="text-2xl text-primary/50">No Messages Yet</h1>
              </div>
            )}
            <div className="flex flex-col gap-2">
              <Input value={msg} onChange={(e) => setMsg(e.target.value)} />
              <Button
                onClick={() => {
                  ws.current?.send(
                    JSON.stringify({
                      type: "send_message",
                      room,
                      message: msg,
                    }),
                  );
                  setMsg("");
                }}
              >
                Send
              </Button>
            </div>
          </div>
        ) : (
          <div className="m-4">
            <Tabs>
              <TabsList defaultValue="create">
                <TabsTrigger value="create">Create</TabsTrigger>
                <TabsTrigger value="join">Join</TabsTrigger>
              </TabsList>
              <TabsContent value="create">
                <div className=" flex flex-col gap-2">
                  <h1 className="text-2xl font-bold mb-6">Create Room</h1>
                  <Label>Enter Room Name</Label>
                  <Input
                    value={roomName}
                    onChange={(e) => setRoomName(e.target.value)}
                  />
                  <Button
                    onClick={() => {
                      if (ws.current) {
                        ws.current.send(
                          JSON.stringify({
                            type: "create_room",
                            room_name: roomName,
                          }),
                        );
                      }
                    }}
                    disabled={roomName === ""}
                  >
                    Send Request
                  </Button>
                </div>
              </TabsContent>
              <TabsContent value="join">
                <div className="flex flex-col gap-2">
                  <h1 className="text-2xl font-bold mb-6">Join Room</h1>
                  <Label>Enter Room Id</Label>
                  <Input
                    value={roomInput}
                    onChange={(e) => setRoomInput(e.target.value)}
                  />
                  <Button
                    onClick={() => {
                      if (ws.current) {
                        ws.current.send(
                          JSON.stringify({ type: "join", room: roomInput }),
                        );
                      }
                    }}
                    disabled={roomInput === ""}
                  >
                    Join
                  </Button>
                </div>
              </TabsContent>
            </Tabs>
          </div>
        )}
      </div>
    </div>
  );
}
