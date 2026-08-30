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

function Voucher({ children }: { children: ReactElement }) {
    const [isOpen, setIsOpen] = useState(false);

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
                            type="text"
                            placeholder="E.g xrm123"
                            className="h-9"
                        />
                        <Button className="w-full cursor-pointer hover:bg-primary/80" >
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