import { useEffect, useState, useRef, useCallback } from "react";

interface WebSocketOptions {
  onOpen?: (event: Event) => void;
  onMessage?: (event: MessageEvent) => void;
  onClose?: (event: CloseEvent) => void;
  onError?: (event: Event) => void;
  reconnectInterval?: number;
  reconnectAttempts?: number;
}

const useWebSocket = (url: string, options?: WebSocketOptions) => {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<MessageEvent | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);

  const connect = useCallback(() => {
    wsRef.current = new WebSocket(url);

    wsRef.current.onopen = (event) => {
      setIsConnected(true);
      reconnectAttemptsRef.current = 0; // Reset on successful connection
      options?.onOpen?.(event);
    };

    wsRef.current.onmessage = (event) => {
      setLastMessage(event);
      options?.onMessage?.(event);
    };

    wsRef.current.onclose = (event) => {
      setIsConnected(false);
      options?.onClose?.(event);
      if (
        options?.reconnectAttempts &&
        reconnectAttemptsRef.current < options.reconnectAttempts
      ) {
        reconnectAttemptsRef.current++;
        setTimeout(connect, options.reconnectInterval || 3000);
      }
    };

    wsRef.current.onerror = (event) => {
      options?.onError?.(event);
      wsRef.current?.close(); // Force close to trigger reconnect logic
    };
  }, [url, options]);

  useEffect(() => {
    connect();

    return () => {
      wsRef.current?.close();
    };
  }, [connect]);

  const sendMessage = useCallback(
    (message: string | object) => {
      if (wsRef.current && isConnected) {
        wsRef.current.send(
          typeof message === "object" ? JSON.stringify(message) : message,
        );
      } else {
        console.warn("WebSocket not connected. Message not sent.");
      }
    },
    [isConnected],
  );

  return { isConnected, lastMessage, sendMessage };
};

export default useWebSocket;
