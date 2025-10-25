import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { ChatLog, Message } from "../components/ChatLog";
import { ChatInput } from "../components/ChatInput";
import "./ChatPage.css";

export function ChatPage() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const navigate = useNavigate();

  const handleSendMessage = async (userMessage: string) => {
    // Add user message to chat
    const newUserMessage: Message = { role: "user", content: userMessage };
    setMessages((prev) => [...prev, newUserMessage]);

    setIsProcessing(true);

    try {
      // Call Tauri command
      const response = await invoke<string>("handle_user_query", {
        query: userMessage
      });

      // Add AI response to chat
      const aiMessage: Message = { role: "assistant", content: response };
      setMessages((prev) => [...prev, aiMessage]);
    } catch (error) {
      console.error("Error processing query:", error);
      const errorMessage: Message = {
        role: "assistant",
        content: `Error: ${error}`
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="chat-page">
      <div className="chat-header">
        <h2>Ask Engine</h2>
        <button
          className="settings-button"
          onClick={() => navigate("/settings")}
          title="Settings"
        >
          ⚙️
        </button>
      </div>
      <div className="chat-content">
        <div className="main-panel">
          <ChatLog messages={messages} />
          <ChatInput
            onSendMessage={handleSendMessage}
            disabled={isProcessing}
          />
        </div>
        <div className="side-panel">
          <div className="webview-placeholder">
            <h3>Browser View</h3>
            <p>AI's browser activity will be displayed here</p>
          </div>
        </div>
      </div>
    </div>
  );
}
