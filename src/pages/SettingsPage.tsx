import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import "./SettingsPage.css";

interface Settings {
  service: string;
  apiKey: string;
  modelName: string;
}

interface GeminiModel {
  name: string;
  displayName: string;
}

interface ModelsResponse {
  models: GeminiModel[];
}

export function SettingsPage() {
  const navigate = useNavigate();
  const [service, setService] = useState("gemini");
  const [apiKey, setApiKey] = useState("");
  const [modelName, setModelName] = useState("");
  const [modelsList, setModelsList] = useState<GeminiModel[]>([]);
  const [isFetchingModels, setIsFetchingModels] = useState(false);
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
        setModelName(settings.modelName || "");
      }
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  };

  const handleFetchModels = async () => {
    if (!apiKey) {
      setMessage("Error: Please enter your API key first");
      return;
    }

    setIsFetchingModels(true);
    setMessage("");

    try {
      const responseJson = await invoke<string>("fetch_gemini_models", {
        apiKey,
      });

      const response: ModelsResponse = JSON.parse(responseJson);

      if (response.models && response.models.length > 0) {
        // Filter to only include models that support generateContent
        const filteredModels = response.models.filter(
          (model) =>
            model.name.includes("gemini") &&
            !model.name.includes("embedding") &&
            !model.name.includes("vision")
        );

        setModelsList(filteredModels);

        // Auto-select first model if none selected
        if (!modelName && filteredModels.length > 0) {
          setModelName(filteredModels[0].name);
        }

        setMessage(`Successfully fetched ${filteredModels.length} models!`);
        setTimeout(() => setMessage(""), 3000);
      } else {
        setMessage("Error: No models found");
      }
    } catch (error) {
      console.error("Failed to fetch models:", error);
      setMessage(`Error: ${error}`);
    } finally {
      setIsFetchingModels(false);
    }
  };

  const handleSave = async () => {
    if (!modelName) {
      setMessage("Error: Please fetch and select a model first");
      return;
    }

    setIsSaving(true);
    setMessage("");

    const settings: Settings = {
      service,
      apiKey,
      modelName,
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
            <label htmlFor="apiKey">Gemini API Key</label>
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

          <button
            className="fetch-button"
            onClick={handleFetchModels}
            disabled={!isFormEnabled || !apiKey || isFetchingModels}
          >
            {isFetchingModels ? "Fetching Models..." : "Fetch Available Models"}
          </button>

          <div className="form-group">
            <label htmlFor="model">Model</label>
            <select
              id="model"
              value={modelName}
              onChange={(e) => setModelName(e.target.value)}
              disabled={!isFormEnabled || modelsList.length === 0}
            >
              {modelsList.length === 0 ? (
                <option value="">Click "Fetch Available Models" first</option>
              ) : (
                modelsList.map((model) => (
                  <option key={model.name} value={model.name}>
                    {model.displayName} ({model.name})
                  </option>
                ))
              )}
            </select>
            {modelsList.length > 0 && (
              <small>{modelsList.length} models available</small>
            )}
          </div>

          <button
            className="save-button"
            onClick={handleSave}
            disabled={!isFormEnabled || isSaving || !apiKey || !modelName}
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
