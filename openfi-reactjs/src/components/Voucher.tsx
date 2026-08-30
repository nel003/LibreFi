import { useState, type ReactElement } from "react";
import { Input } from "./ui/input";
import { Button } from "./ui/button";
import {
    Dialog,
    DialogTrigger,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription
} from "../components/ui/dialog";
import { toast } from "./ui/toast";

function Voucher({ updateUser, children }: { updateUser: (expires_on: number, now: number) => void; children: ReactElement }) {
    const [isOpen, setIsOpen] = useState(false);
    const [isRedeeming, setIsRedeeming] = useState(false);
    const [code, setCode] = useState("");

    async function handleRedeem() {
        const redeemRequest = async () => {
            setIsRedeeming(true);
            const res = await fetch("http://localhost:8000/redeem", {
                method: "POST",
                body: JSON.stringify({ code })
            })

            if (!res.ok) {
                let serverErrorMessage = "Failed to update status";

                try {
                    const errorData = await res.json();
                    serverErrorMessage = errorData.error || serverErrorMessage;
                } catch {
                    const textError = await res.text();
                    if (textError) serverErrorMessage = textError;
                }

                setIsRedeeming(false);
                throw new Error(serverErrorMessage);
            }


            const json = await res.json();
            updateUser(json.expires_on, json.now)
            setIsRedeeming(false);
            setIsOpen(false)
        }

        toast.promise(
            redeemRequest(),
            {
                loading: {
                    title: "Processing...",
                    description: "Please wait while we process your voucher."
                },
                success: {
                    title: "Success",
                    description: "Voucher applied successfully."
                },
                error: (err) => ({
                    title: "Update failed",
                    description: err instanceof Error ? err.message : "Something went wrong."
                })
            },
            { position: "top-center" }
        );
    }

    return (
        <>
            <Dialog open={isOpen} onOpenChange={setIsOpen}>
                <DialogTrigger render={children}>
                    Subscribe
                </DialogTrigger>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Redeem Voucher</DialogTitle>
                        <DialogDescription>
                            Enter your voucher and click redeem.
                        </DialogDescription>
                    </DialogHeader>
                    <div className="flex flex-col gap-3">
                        <Input
                            onChange={(e) => setCode(e.target.value)}
                            value={code}
                            type="text"
                            placeholder="E.g xrm123"
                            className="h-9"
                        />
                        <Button disabled={isRedeeming || !code.trim()} onClick={handleRedeem} className="w-full cursor-pointer hover:bg-primary/80" >
                            Redeem Now
                        </Button>
                        <Button onClick={() => setIsOpen(false)} variant="ghost" className="w-full cursor-pointer -mt-2" >
                            Cancel
                        </Button>
                    </div>
                </DialogContent>
            </Dialog>
        </>
    )
}

export default Voucher;