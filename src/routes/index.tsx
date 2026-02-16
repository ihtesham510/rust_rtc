import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useReducer, useRef, useState } from "react";

export const Route = createFileRoute("/")({
	component: App,
});

type Reducer = Room | null;
type ReducerAction =
	| { type: "set_room"; room: Reducer }
	| { type: "add_message"; message: { by: string; message: string } }
	| { type: "set_messages"; messages: Array<{ by: string; message: string }> };

function App() {
	function reducer(state: Reducer, action: ReducerAction): Reducer {
		switch (action.type) {
			case "set_room":
				return action.room;
			case "add_message":
				if (state) {
					return {
						...state,
						messages: [...state.messages, action.message],
					};
				}
				return state;
			case "set_messages":
				if (state) {
					return {
						...state,
						messages: action.messages,
					};
				}
				return state;
			default:
				return state;
		}
	}
	const [room, dispatchRoom] = useReducer(reducer, null);
	const roomRef = useRef<Room | null>(null);

	// Keep roomRef in sync with room state
	useEffect(() => {
		roomRef.current = room;
	}, [room]);

	const [connected, setIsConnected] = useState(false);
	const [userId, setUserId] = useState<string | null>(null);
	const [availableRooms, setAvailableRooms] = useState<AvailableRooms[] | null>(
		null,
	);
	const [roomInput, setRoomInput] = useState("");
	const [roomName, setRoomName] = useState("");
	const [msg, setMsg] = useState<string>("");
	const ws = useRef<WebSocket | null>(null);

	useEffect(() => {
		if (connected && ws.current) {
			ws.current.send(JSON.stringify({ type: "info" }));
			ws.current.send(JSON.stringify({ type: "get_rooms" }));
		}
	}, [connected]);

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
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
					dispatchRoom({
						type: "set_room",
						room: null,
					});
					setUserId(null);
				};
				ws.current.onmessage = (event) => {
					const data = JSON.parse(event.data);
					console.log(data);

					// Handle array response from get_rooms
					if (Array.isArray(data)) {
						const rooms: AvailableRooms[] = data.map((room: Room) => ({
							room_id: room.room,
							room_name: room.room_name,
						}));
						setAvailableRooms(rooms);
						return;
					}

					const message = data as Messages;
					switch (message.type) {
						case "room_available":
							console.log("room_available");
							setAvailableRooms((prev) =>
								prev
									? [
											...prev,
											{
												room_id: message.room_id,
												room_name: message.room_name,
											},
										]
									: [
											{
												room_id: message.room_id,
												room_name: message.room_name,
											},
										],
							);
							break;
						case "room_created":
							dispatchRoom({
								type: "set_room",
								room: {
									room_name: message.room_name,
									room: message.room_id,
									messages: [],
									users: [],
									admin: userId || "",
								},
							});
							break;
						case "room_joined":
							dispatchRoom({
								type: "set_room",
								room: {
									room_name: message.room_name,
									room: message.room_id,
									messages: [],
									users: [],
									admin: "",
								},
							});
							ws.current?.send(
								JSON.stringify({
									type: "list_messages",
									room: message.room_id,
								}),
							);
							break;
						case "list_messages":
							dispatchRoom({
								type: "set_messages",
								messages: message.messages,
							});
							break;
						case "rooms_available":
							setAvailableRooms(
								message.rooms.map(
									(room: { room: string; room_name: string }) => {
										return {
											room_id: room.room,
											room_name: room.room_name,
										} satisfies AvailableRooms;
									},
								),
							);
							break;
						case "info":
							setUserId(message.user_id);
							break;
						case "room_broadcast":
							dispatchRoom({
								type: "add_message",
								message: {
									by: message.by,
									message: message.message,
								},
							});
							break;
					}
				};
			}
		})();
		return () => ws.current?.close();
	}, []);

	function joinRoom(id: string) {
		ws.current?.send(JSON.stringify({ type: "join", room: id }));
	}

	function leaveRoom() {
		if (room && userId) {
			ws.current?.send(
				JSON.stringify({
					type: "leave_room",
					room: room.room,
					user: userId,
				}),
			);
			dispatchRoom({
				type: "set_room",
				room: null,
			});
		}
	}

	function getRoomDetails() {
		if (room) {
			ws.current?.send(
				JSON.stringify({
					type: "get_room",
					room: room.room,
				}),
			);
		}
	}

	return (
		<div className="m-6">
			{connected ? "connected" : "disconnected"}
			<div>
				{room ? (
					<div className="p-4 h-screen w-full grid space-y-4">
						<div className="w-full flex items-center justify-between">
							<h1 className="text-2xl font-bold">{room.room_name}</h1>
							<div className="flex gap-2 items-center">
								<p className="text-sm text-muted-foreground">{room.room}</p>
								<Button variant="outline" onClick={getRoomDetails}>
									Refresh
								</Button>
								<Button variant="destructive" onClick={leaveRoom}>
									Leave Room
								</Button>
							</div>
						</div>
						{room.messages && room.messages.length > 0 ? (
							<ScrollArea className="border-border border rounded-md p-2 min-h-[400px] max-h-[800px] flex flex-col space-y-4">
								{room.messages.map((msg, index) => (
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
									if (room) {
										ws.current?.send(
											JSON.stringify({
												type: "send_message",
												room: room.room,
												message: msg,
											}),
										);
										setMsg("");
									}
								}}
								disabled={!msg.trim()}
							>
								Send
							</Button>
						</div>
					</div>
				) : (
					<div className="m-4">
						<Tabs defaultValue="create">
							<TabsList>
								<TabsTrigger value="create">Create</TabsTrigger>
								<TabsTrigger value="join">Join</TabsTrigger>
								<TabsTrigger value="rooms">Rooms</TabsTrigger>
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
							<TabsContent value="rooms">
								{availableRooms && availableRooms.length !== 0 ? (
									availableRooms.map((room) => (
										<div
											className="flex justify-between border-border border p-4 rounded-md"
											key={room.room_id}
										>
											<h1 className="text-xl font-bold">{room.room_name}</h1>
											<Button
												onClick={() => {
													joinRoom(room.room_id);
												}}
											>
												Join
											</Button>
										</div>
									))
								) : (
									<div>no rooms avaialbe</div>
								)}
							</TabsContent>
						</Tabs>
					</div>
				)}
			</div>
		</div>
	);
}
