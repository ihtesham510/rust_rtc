interface AvailableRooms {
	room_id: string;
	room_name: string;
}
interface RoomMessage {
	by: string;
	message: string;
}
interface Room {
	room_name: string;
	room: string;
	messages: Array<RoomMessage>;
	users: Array<string>;
	admin: string;
}

interface RoomLeft {
	type: "room_left";
	room: Room;
}
interface RoomsAvailable {
	type: "rooms_available";
	rooms: Array<Room>;
}
interface ListMessages {
	type: "list_messages";
	messages: Array<RoomMessage>;
}
interface RoomBroadcast extends RoomMessage {
	type: "room_broadcast";
}
interface RoomCreated {
	type: "room_created";
	room_id: string;
	room_name: string;
}
interface RoomJoined {
	type: "room_joined";
	room_id: string;
	room_name: string;
}
interface RoomAvailable {
	type: "room_available";
	room_id: string;
	room_name: string;
}

interface Info {
	type: "info";
	user_id: string;
}

type Messages =
	| RoomLeft
	| RoomsAvailable
	| ListMessages
	| RoomBroadcast
	| RoomCreated
	| RoomJoined
	| RoomAvailable
	| Info;
