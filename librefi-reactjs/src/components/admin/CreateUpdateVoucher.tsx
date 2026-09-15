import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "#components/ui/popover"
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

export default function CreateEditVoucher({ getVouchers, code, price, time, btn }: { getVouchers: () => Promise<void>, code?: string, price?: number, time?: number, btn: React.ReactElement }) {
    const [fields, setFields] = useState<{ code?: string, price?: number, time?: number }>()
    const [loading, setLoading] = useState(false);
    const [open, setOpen] = useState(false);
    const [dialog, setDialog] = useState(false);
    const [newVoucher, setNewVocuher] = useState("");

    useEffect(() => {
        setFields({ code, price: price ? price : 0, time: time ? time : 0 })
    }, [])

    async function updateEdit() {
        setLoading(true)

        if (fields?.price && fields?.time) {
            const res = await fetch("http://localhost:8000/api/admin/vouchers", {
                method: "POST",
                body: JSON.stringify({ payload: await encrypt(getAdminKey(), JSON.stringify({ code: code ? code : "RANDOM", price: fields.price, time: fields.time, used: false })) })
            })
            if (res.ok) {
                const json = await res.json();
                toast.success("Success", {
                    description: code ? `Voucher ${code} updated successfully.` : `Generated successfully. View it by clicking the latest code.`,
                    position: "top-center"
                })
                setNewVocuher(json.code)
                setOpen(false);
                if (!fields?.code) setDialog(true)
                await getVouchers();
            } else {
                toast.error("Error", {
                    description: code ? `Failed to update voucher ${code}.` : `Failed to generate voucher.`,
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
                        <h4 className="m-0 font-semibold">{code ? "Edit Voucher" : "New Voucher"}</h4>
                        <p className="text-muted-foreground">{code ? "View and update voucher details." : "Generate a new voucher"}</p>
                    </div>
                    <div className="space-y-3 p-3 pb-4">
                        <div className="space-y-1.5">
                            <Label>Code</Label>
                            <Input disabled value={code ? code : "RANDOM"} />
                        </div>
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
                        <Button onClick={updateEdit} disabled={!fields?.price || !fields?.time} className="w-full">{code ? "Update" : "Generate"} {loading ? <Loader2Icon className="animate-spin" /> : <></>}</Button>
                    </div>
                </PopoverContent>
            </Popover>

            <Dialog open={dialog} onOpenChange={setDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle className="p-4 pb-0 flex justify-center text-primary"><CheckCircle2 size={40} /></DialogTitle>
                        <h1 className="text-center -mt-1 text-foreground/60">Here is you voucher code:</h1>
                        <div className="w-full text-center font-semibold text-xl py-2">
                            {newVoucher}
                        </div>
                        <p className="text-center text-foreground/60">
                            Your new voucher has been generated successfully and is now ready to use.
                        </p>
                    </DialogHeader>
                </DialogContent>
            </Dialog>
        </div>
    )
}
