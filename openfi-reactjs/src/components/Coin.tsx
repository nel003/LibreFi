import { useEffect, useRef, useState, type ReactElement } from "react";
import {
    Dialog,
    DialogTrigger,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription
} from "../components/ui/dialog";
import CircularProgress from "./ui/circleProgress";
import { Button } from "./ui/button";
import { toast } from "./ui/toast";

function Progress({ MAX, timeSec, id, closeModal }: { MAX: number; timeSec: number; id: number; closeModal: () => void }) {
    const [progress, setProgress] = useState(100);
    const timeRef = useRef<number | null>(null);
    const secRef = useRef<number>(timeSec);

    useEffect(() => {
        if (timeRef.current) {
            clearInterval(timeRef.current)
            timeRef.current = null;
            secRef.current = timeSec;
            setProgress(100)
        }

        timeRef.current = setInterval(() => {
            secRef.current -= 100;
            setProgress(secRef.current / MAX * 100)
            if (secRef.current <= 0 && timeRef.current) {
                clearInterval(timeRef.current)
                timeRef.current = null;
                closeModal();
            }
        }, 100);

        console.log(timeRef.current)


        return () => {
            if (timeRef.current) {
                clearInterval(timeRef.current)
                timeRef.current = null;
            }
        }
    }, [secRef, timeSec, MAX, closeModal, id]);

    return (<>
        <div className="relative flex justify-center w-full">
            <CircularProgress progressClassName={`opacity-60 ${progress <= 25 ? "stroke-yellow-400" : "stroke-primary"}`} size={250} strokeWidth={15} value={progress} />
            <div className="w-full h-full grid place-items-center absolute top-0 left-0">
                <h1 className="font-medium text-5xl text-foreground/70">₱0</h1>
            </div>
        </div>
    </>)
}

function Coin({ children }: { children: ReactElement }) {
    const [isOpen, setIsOpen] = useState(false);
    const [timeSec] = useState(30_000);
    const [id] = useState(1);

    function closeModal() {
        setIsOpen(false);
    }

    async function handleCoin() {
        const res = await fetch("http://localhost:8000/coin");
        const json = await res.json();

        if (!res.ok) {
            toast.error("Coinslot is not available!", {
                description: json.error,
                position: "top-center"
            })
            return;
        }
        setIsOpen(true);
    }

    return (
        <Dialog open={isOpen}>
            <DialogTrigger onClick={handleCoin} render={children}>
                Subscribe
            </DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <span className="absolute right-2 top-2 block h-6 w-6 bg-popover z-40" />
                    <DialogTitle>Insert Coins</DialogTitle>
                    <DialogDescription>
                        Click done after inserting coins.
                    </DialogDescription>
                </DialogHeader>
                <div className="flex flex-col gap-3">

                    <Progress MAX={30_000} timeSec={timeSec} id={id} closeModal={closeModal} />

                    <Button onClick={() => setIsOpen(false)} className="w-full cursor-pointer hover:bg-primary/80" >
                        Done
                    </Button>
                    {/* <Button onClick={() => setIsOpen(false)} variant="secondary" className="w-full cursor-pointer -mt-2" >
                            Cancel
                        </Button> */}
                </div>
            </DialogContent>
        </Dialog>
    )
}

export default Coin;