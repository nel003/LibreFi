import Coin from "#components/Coin";
import More from "#components/More";
import Timer from "#components/Timer";
import { Button } from "#components/ui/button"
import Voucher from "#components/Voucher";
import { CoinsIcon, EllipsisVertical, PauseIcon, PlayIcon, Ticket } from "lucide-react"
import { useState } from "react"

function App() {
  const [isPaused, setIsPaused] = useState(false);

  return (
    <>
      <div className="grid place-items-center h-screen w-screen">
        <div className="max-w-lg w-full p-6">
          <h1 className="text-center pb-8 text-sm">CONNECTED</h1>

          <div className="bg-amber-50/30 dark:bg-amber-300/5">
            <div className="w-full flex justify-center backdrop-blur-3xl  pt-4">
              <Timer isPaused={isPaused} timeSec={12333} />
            </div>
          </div>

          <div className="w-full flex gap-4 pt-16 justify-center">
            <div className="flex flex-col">
              <Button onClick={() => setIsPaused(!isPaused)} variant="outline" size="icon-lg" className="hover:scale-125 hover:bg-primary/5 hover:text-primary/90 hover:-rotate-6">
                {isPaused ? <PlayIcon className="text-green-500" /> : <PauseIcon className="text-yellow-400" />}
              </Button>
              <span className="text-[0.7rem] text-center mt-1 text-foreground/60">{isPaused ? "PLAY" : "PAUSE"}</span>
            </div>
            <div className="flex flex-col">
              <Coin>
                <Button variant="outline" size="icon-lg" className="hover:scale-125 hover:bg-primary/5 hover:text-primary/90 hover:-rotate-6">
                  <CoinsIcon />
                </Button>
              </Coin>
              <span className="text-[0.7rem] text-center mt-1 text-foreground/60">COIN</span>
            </div>
            <div className="flex flex-col">
              <Voucher>
                <Button variant="outline" size="icon-lg" className="hover:scale-125 hover:bg-primary/5 hover:text-primary/90 hover:-rotate-6">
                <Ticket />
              </Button>
              </Voucher>
              <span className="text-[0.7rem] text-center mt-1 text-foreground/60">VOUC</span>
            </div>
            <div className="flex flex-col">
              <More>
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
