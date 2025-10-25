import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import "./SettingsPage.css";

interface Settings {
  service: string;
  apiKey: string;
  model: string;
}

const GEMINI_MODELS = [
  "gemini-1.5-pro",
  "gemini-1.5-flash",
  "gemini-1.5-flash-latest",
  "gemini-pro",
];

export function SettingsPage() {
  const navigate = useNavigate();
  const [service, setService] = useState("gemini");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("gemini-1.5-flash");
  const [isSaving, setIsSaving] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const settingsJson = await invoke<string>("load_settings");
      if (settingsJson && settingsJson !== "{}") {
        const settings: Settings = JSON.parse(settingsJson);
        setService(settings.service || "gemini");
        setApiKey(settings.apiKey || "");
        setModel(settings.model || "gemini-1.5-flash");
      }
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    setMessage("");

    const settings: Settings = {
      service,
      apiKey,
      model,
    };

    try {
      await invoke("save_settings", {
        settings: JSON.stringify(settings),
      });
      setMessage("Settings saved successfully!");
      setTimeout(() => setMessage(""), 3000);
    } catch (error) {
      console.error("Failed to save settings:", error);
      setMessage(`Error: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  const isFormEnabled = service === "gemini";

  return (
    <div className="settings-page">
      <div className="settings-header">
        <button className="back-button" onClick={() => navigate("/")}>
          ← Back to Chat
        </button>
        <h2>Settings</h2>
      </div>

      <div className="settings-content">
        <div className="settings-form">
          <div className="form-group">
            <label htmlFor="service">AI Provider</label>
            <select
              id="service"
              value={service}
              onChange={(e) => setService(e.target.value)}
            >
              <option value="gemini">Google Gemini</option>
              <option value="openai">OpenAI</option>
              <option value="anthropic">Anthropic Claude</option>
            </select>
          </div>

          {!isFormEnabled && (
            <div className="coming-soon">
              Coming Soon: {service === "openai" ? "OpenAI" : "Anthropic Claude"}{" "}
              support is under development.
            </div>
          )}

          <div className="form-group">
            <label htmlFor="apiKey">
              {service === "gemini" ? "Gemini" : service === "openai" ? "OpenAI" : "Anthropic"} API Key
            </label>
            <input
              id="apiKey"
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="Enter your API key"
              disabled={!isFormEnabled}
            />
            {service === "gemini" && (
              <small>
                Get your API key from{" "}
                <a
                  href="https://aistudio.google.com/app/apikey"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Google AI Studio
                </a>
              </small>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="model">Model</label>
            <select
              id="model"
              value={model}
              onChange={(e) => setModel(e.target.value)}
              disabled={!isFormEnabled}
            >
              {GEMINI_MODELS.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          </div>

          <button
            className="save-button"
            onClick={handleSave}
            disabled={!isFormEnabled || isSaving || !apiKey}
          >
            {isSaving ? "Saving..." : "Save Settings"}
          </button>

          {message && (
            <div
              className={`message ${
                message.startsWith("Error") ? "error" : "success"
              }`}
            >
              {message}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
