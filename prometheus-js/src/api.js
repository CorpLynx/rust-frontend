import axios from 'axios';

export class OllamaAPI {
  constructor(baseUrl, timeout = 30000) {
    this.baseUrl = baseUrl.replace(/\/$/, '');
    this.timeout = timeout;
  }

  async fetchModels() {
    try {
      const response = await axios.get(`${this.baseUrl}/api/tags`, {
        timeout: this.timeout
      });

      if (response.data.models && Array.isArray(response.data.models)) {
        return response.data.models.map(m => m.name || m.id);
      }
      
      return [];
    } catch (error) {
      throw new Error(`Failed to fetch models: ${error.message}`);
    }
  }

  async *generateStream(model, prompt) {
    try {
      const response = await axios.post(
        `${this.baseUrl}/api/generate`,
        {
          model,
          prompt,
          stream: true
        },
        {
          timeout: this.timeout,
          responseType: 'stream'
        }
      );

      let buffer = '';
      
      for await (const chunk of response.data) {
        buffer += chunk.toString();
        
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';
        
        for (const line of lines) {
          if (line.trim()) {
            try {
              const json = JSON.parse(line);
              if (json.response) {
                yield json.response;
              }
              if (json.done) {
                return;
              }
            } catch (e) {
              // Skip invalid JSON
            }
          }
        }
      }
    } catch (error) {
      throw new Error(`Stream error: ${error.message}`);
    }
  }

  async generate(model, prompt) {
    let fullResponse = '';
    
    for await (const chunk of this.generateStream(model, prompt)) {
      fullResponse += chunk;
    }
    
    return fullResponse;
  }
}
