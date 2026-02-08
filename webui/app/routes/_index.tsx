import { useEffect, useState } from "react";
import { Workstation } from "~/components/Workstation";

export default function Index() {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-chassis-950">
        <p className="font-mono text-sm text-oled-dim">WAVELET</p>
      </div>
    );
  }

  return <Workstation />;
}
