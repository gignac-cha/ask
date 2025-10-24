import { useEffect, useRef } from "react";
import "./ChatLog.css";

export interface Message {
  role: "user" | "assistant";
  content: string;
}

interface ChatLogProps {
  messages: Message[];
}

export function ChatLog({ messages }: ChatLogProps) {
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  return (
    <div className="chat-log">
      {messages.length === 0 ? (
        <div className="chat-log-empty">
          <h2>Ask Engine</h2>
          <p>Ask me anything about the web...</p>
        </div>
      ) : (
        messages.map((message, index) => (
          <div key={index} className={`message message-${message.role}`}>
            <div className="message-role">
              {message.role === "user" ? "You" : "AI"}
            </div>
            <div className="message-content">{message.content}</div>
          </div>
        ))
      )}
      <div ref={logEndRef} />
    </div>
  );
}
