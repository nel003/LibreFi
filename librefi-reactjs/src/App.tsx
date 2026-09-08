import Coin from "#components/Coin";
import More from "#components/More";
import Timer from "#components/Timer";
import { Button } from "#components/ui/button"
import Voucher from "#components/Voucher";
import { CoinsIcon, EllipsisVertical, Loader2Icon, PauseIcon, PlayIcon, Ticket } from "lucide-react"
import { useCallback, useEffect, useState } from "react"
import type { UserType } from "./types/user";
import { toast } from "#components/ui/toast";

function App() {
  const [user, setUser] = useState<UserType>();
  const [status, setStatus] = useState("--");

  const init = useCallback(async () => {
    const res = await fetch(`/init?_=${Date.now()}`, {
      method: "GET"
    });

    if (res.status === 200) {
      const json = await res.json();
      setUser(json)
    }

  }, []);

  const testConnection = useCallback(async () => {
    try {
      setStatus("Testing");
      const res = await fetch(`https://dns.google/resolve?name=google.com&_=${Date.now()}`, {
        method: "GET",
        signal: AbortSignal.timeout(2000)
      });

      if (res.status === 200) {
        setStatus("Connected");
      }
    } catch (error) {
      setStatus("Disconnected");
    }

  }, []);

  useEffect(() => {
    (() => {
      init();
    })()
  }, [init])

  useEffect(() => {
    if (user && window.location.pathname !== "/") {
      window.location.reload();
      return;
    }

    (async () => {
      if (user?.paused) {
        setStatus("Paused")
        return;
      }

      await testConnection();
    })()
  }, [user])

  async function handlePlayPause() {
    const playPauseRequest = async () => {
      const res = await fetch("/play_pause", {
        method: "POST"
      });

      if (!res.ok) {
        let serverErrorMessage = "Failed to update status";

        try {
          const errorData = await res.json();
          serverErrorMessage = errorData.error || serverErrorMessage;
        } catch {
          const textError = await res.text();
          if (textError) serverErrorMessage = textError;
        }

        throw new Error(serverErrorMessage);
      }

      const json = await res.json();

      if (user) {
        setUser({
          ...user,
          expires_on: json.expires_on,
          now: json.now,
          pause_attempts: json.pause_attempts,
          paused: json.paused,
          paused_on: json.paused_on
        });
        return json;
      } else {
        throw new Error("User session not found.");
      }
    };

    toast.promise(
      playPauseRequest(),
      {
        loading: {
          title: "Updating status...",
          description: "Please wait while we update your connection."
        },
        success: {
          title: "Success",
          description: "Status has been updated successfully."
        },
        error: (err) => ({
          title: "Update failed",
          description: err instanceof Error ? err.message : "Something went wrong."
        })
      },
      { position: "top-center" }
    );
  }

  function updateUser(expires_on: number, now: number) {
    if (user) setUser({ ...user, expires_on: expires_on, now: now })
  }

  return (
    <>
      <div className="grid place-items-center h-screen [@supports(height:100dvh)]:h-dvh w-screen">
        <div className="max-w-lg w-full p-6">
          <h1 className="text-center pb-8 text-sm flex items-center justify-center ">{status} {status === "Testing" ? <span className="pl-1 "> <Loader2Icon size={16} className="animate-spin " /></span> : <></>} </h1>

          <div className="bg-amber-50/30 dark:bg-amber-300/5">
            <div className="w-full flex justify-center backdrop-blur-3xl  py-4">
              <Timer
                user={user}
              />
            </div>
          </div>

          <div className="w-full flex gap-4 pt-16 justify-center">
            <div className="flex flex-col">
              <Button onClick={handlePlayPause} variant="outline" size="icon-lg" className="hover:scale-125 hover:bg-primary/5 hover:text-primary/90 hover:-rotate-6">
                {user?.paused ? <PlayIcon className="text-green-500" /> : <PauseIcon className="text-yellow-400" />}
              </Button>
              <span className="text-[0.7rem] text-center mt-1 text-foreground/60">{user?.paused ? "PLAY" : "PAUSE"}</span>
            </div>
            <div className="flex flex-col">
              <Coin onClose={init}>
                <Button variant="outline" size="icon-lg" className="hover:scale-125 hover:bg-primary/5 hover:text-primary/90 hover:-rotate-6">
                  <CoinsIcon />
                </Button>
              </Coin>
              <span className="text-[0.7rem] text-center mt-1 text-foreground/60">COIN</span>
            </div>
            <div className="flex flex-col">
              <Voucher updateUser={updateUser}>
                <Button variant="outline" size="icon-lg" className="hover:scale-125 hover:bg-primary/5 hover:text-primary/90 hover:-rotate-6">
                  <Ticket />
                </Button>
              </Voucher>
              <span className="text-[0.7rem] text-center mt-1 text-foreground/60">VOUC</span>
            </div>
            <div className="flex flex-col">
              <More user={user}>
                <Button variant="outline" size="icon-lg" className="hover:scale-125 hover:bg-primary/5 hover:text-primary/90 hover:-rotate-6">
                  <EllipsisVertical />
                </Button>
              </More>
              <span className="text-[0.7rem] text-center mt-1 text-foreground/60">MORE</span>
            </div>
          </div>

        </div>
      </div>
    </>
  )
}

export default App
