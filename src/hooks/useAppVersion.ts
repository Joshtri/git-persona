import { useEffect, useState } from "react";
import { appVersion } from "@/ipc/client";

export function useAppVersion(): string {
  const [version, setVersion] = useState("…");

  useEffect(() => {
    appVersion()
      .then(setVersion)
      .catch(() => {});
  }, []);

  return version;
}
