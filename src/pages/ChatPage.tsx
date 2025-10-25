import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useNavigate } from "react-router-dom";
import { ChatLog, Message } from "../components/ChatLog";
import { ChatInput } from "../components/ChatInput";
import "./ChatPage.css";

// Event payload types
interface AutomationStatus {
  message: string;
  step: number;
  total_steps: number;
  current_url: string | null;
}

interface AutomationComplete {
  success: boolean;
  answer: string | null;
  error: string | null;
}

interface BrowserViewUpdate {
  screenshot: string; // base64
  url: string;
}

export function ChatPage() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [browserScreenshot, setBrowserScreenshot] = useState<string | null>(null);
  const [browserUrl, setBrowserUrl] = useState<string>("");
  const navigate = useNavigate();

  // Set up event listeners
  useEffect(() => {
    // Listen for automation status updates
    const unlistenStatusPromise = listen<AutomationStatus>("automation-status", (event) => {
      const payload = event.payload;
      const statusMessage = `[${payload.step}/${payload.total_steps}] ${payload.message}`;
      setMessages((prev) => [
        ...prev.slice(-99), // Keep last 99 messages to limit history
        { role: "assistant", content: statusMessage }
      ]);
    });

    // Listen for automation completion
    const unlistenCompletePromise = listen<AutomationComplete>("automation-complete", (event) => {
      const payload = event.payload;
      if (payload.success && payload.answer) {
        setMessages((prev) => [
          ...prev.slice(-99),
          { role: "assistant", content: payload.answer! }
        ]);
      } else if (payload.error) {
        setMessages((prev) => [
          ...prev.slice(-99),
          { role: "assistant", content: `Error: ${payload.error}` }
        ]);
      }
      setIsProcessing(false);
    });

    // Listen for browser view updates
    const unlistenBrowserPromise = listen<BrowserViewUpdate>("browser-view-update", (event) => {
      const payload = event.payload;
      setBrowserScreenshot(`data:image/png;base64,${payload.screenshot}`);
      setBrowserUrl(payload.url);
    });

    // Cleanup listeners on unmount
    return () => {
      unlistenStatusPromise.then((fn) => fn());
      unlistenCompletePromise.then((fn) => fn());
      unlistenBrowserPromise.then((fn) => fn());
    };
  }, []);

  const handleSendMessage = async (userMessage: string) => {
    // Add user message to chat
    const newUserMessage: Message = { role: "user", content: userMessage };
    setMessages((prev) => [...prev, newUserMessage]);

    // Clear browser screenshot for new task
    setBrowserScreenshot(null);
    setBrowserUrl("");
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
          {browserScreenshot ? (
            <div className="browser-view">
              <div className="browser-url">{browserUrl}</div>
              <img src={browserScreenshot} alt="Browser view" />
            </div>
          ) : (
            <div className="webview-placeholder">
              <h3>Browser View</h3>
              <p>AI's browser activity will appear here</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
