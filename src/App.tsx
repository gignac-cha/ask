import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChatLog, Message } from "./components/ChatLog";
import { ChatInput } from "./components/ChatInput";
import "./App.css";

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);

  const handleSendMessage = async (userMessage: string) => {
    // Add user message to chat
    const newUserMessage: Message = { role: "user", content: userMessage };
    setMessages((prev) => [...prev, newUserMessage]);

    setIsProcessing(true);

    try {
      // Call Tauri command (will implement in P0-2)
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
    <div className="app-container">
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
  );
}

export default App;
