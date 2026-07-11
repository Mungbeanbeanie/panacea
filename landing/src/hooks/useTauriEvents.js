// useTauriEvents — subscribes to a Rust→UI event stream by name and returns its latest
// payload (null until the first event, or if the stream drops). The single data-access
// point for the live views; components stay presentational.
import { useEffect, useState } from "react";

export function useTauriEvents(stream) {
  const [data, setData] = useState(null);
  useEffect(() => {
    // TODO(M7): wire to @tauri-apps/api `listen(stream, (e) => setData(e.payload))`
    // and return the unlisten fn for cleanup.
    return () => {};
  }, [stream]);
  return data;
}
