import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "../ui/popover"
import { CheckCircle2, Loader2Icon } from "lucide-react";
import { Label } from "../ui/label";
import { Input } from "../ui/input";
import { toast } from "../ui/toast";
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
} from "../ui/dialog"
import { useEffect, useState } from "react";
import { encrypt, formatSeconds, getAdminKey } from "#lib/utils";
import { Button } from "../ui/button";

export default function CreateUpdateRate({ getRates, id, price, time, btn }: { getRates: () => Promise<void>, id?: number, price?: number, time?: number, btn: React.ReactElement }) {
    const [fields, setFields] = useState<{ price?: number, time?: number }>()
    const [loading, setLoading] = useState(false);
    const [open, setOpen] = useState(false);
    const [dialog, setDialog] = useState(false);

    useEffect(() => {
        setFields({ price: price ? price : 0, time: time ? time : 0 })
    }, [])

    async function updateEdit() {
        setLoading(true)

        if (fields?.price && fields?.time) {
            const payload = id ? { id, price: fields.price, time: fields.time } : { price: fields.price, time: fields.time }
            const res = await fetch("http://localhost:8000/api/admin/rates", {
                method: id ? "PUT" : "POST",
                body: JSON.stringify({ payload: await encrypt(getAdminKey(), JSON.stringify(payload)) })
            })
            if (res.ok) {
                toast.success("Success", {
                    description: id ? `Rate updated successfully.` : `Rate created successfully.`,
                    position: "top-center"
                })
                setOpen(false);
                if (!id) setDialog(true)
                await getRates();
            } else {
                toast.error("Error", {
                    description: id ? `Failed to update rate.` : `Failed to create rate.`,
                    position: "top-center"
                })
            }
        }
        setLoading(false)
    }

    return (
        <div>
            <Popover open={open} onOpenChange={setOpen}>
                <PopoverTrigger render={btn}>
                    <div aria-hidden="true" />
                </PopoverTrigger>
                <PopoverContent className="w-72 gap-0 p-0" align="end">
                    <div className="border-b p-3">
                        <h4 className="m-0 font-semibold">{id ? "Edit Rate" : "New Rate"}</h4>
                        <p className="text-muted-foreground">{id ? "View and update rate details." : "Create a new rate"}</p>
                    </div>
                    <div className="space-y-3 p-3 pb-4">
                        <div className="space-y-1.5">
                            <Label>Price</Label>
                            <Input type="number" placeholder="E.g 1" onChange={(e) => fields && setFields({ ...fields, price: parseInt(e.target.value) })} value={fields?.price} />
                        </div>
                        <div className="space-y-1.5">
                            <Label>Time<span className="text-foreground/60">(seconds)</span></Label>
                            <Input type="number" placeholder="E.g 900" onChange={(e) => fields && setFields({ ...fields, time: parseInt(e.target.value) })} value={fields?.time} />
                            <p className="text-xs">{formatSeconds(fields?.time || 0)}</p>
                        </div>
                    </div>

                    <div className="p-2 w-full">
                        <Button onClick={updateEdit} disabled={!fields?.price || !fields?.time} className="w-full">{id ? "Update" : "Create"} {loading ? <Loader2Icon className="animate-spin" /> : <></>}</Button>
                    </div>
                </PopoverContent>
            </Popover>

            <Dialog open={dialog} onOpenChange={setDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle className="p-4 pb-0 flex justify-center text-primary"><CheckCircle2 size={40} /></DialogTitle>
                        <h1 className="text-center font-semibold text-xl py-2">Success</h1>
                        <p className="text-center text-foreground/60">
                            Your new rate has been created successfully.
                        </p>
                    </DialogHeader>
                </DialogContent>
            </Dialog>
        </div>
    )
}
