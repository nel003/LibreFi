import { Button } from "#components/ui/button";
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "#components/ui/dialog";
import { toast } from "#components/ui/toast";
import { encrypt, getAdminKey } from "#lib/utils";
import { Loader2, Trash2 } from "lucide-react";
import { useState } from "react";

export default function DeleteVoucher({ code, getVouchers }: { code: string, getVouchers: () => Promise<void> }) {
    const [loading, setLoading] = useState(false)
    const [open, setOpen] = useState(false)

    async function handleDelete() {
        setLoading(true)
        const res = await fetch("http://localhost:8000/api/admin/vouchers", {
            method: "DELETE",
            body: JSON.stringify({ payload: await encrypt(getAdminKey(), JSON.stringify({ code })) })
        })

        if (res.ok) {
            toast.success("Success", {
                description: `Voucher ${code} has been deleted successfully.`,
                position: "top-center"
            })
        } else {
            toast.error("Error", {
                description: `Failed to delete voucher ${code}.`,
                position: "top-center"
            })
        }
        await getVouchers();
        setOpen(false)
        setLoading(false)
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger render={<Button size="icon-xs" variant="outline" className="border-amber-300 text-foreground/60"><Trash2 /></Button>}></DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>Are you absolutely sure?</DialogTitle>
                    <DialogDescription>
                        This action cannot be undone. This will permanently delete the voucher code: <span className="font-semibold">{code}</span>
                    </DialogDescription>
                </DialogHeader>
                <div className="flex gap-2 w-full">
                    <DialogClose className="grow" render={<Button variant="outline">Cancel</Button>} />
                    <Button onClick={handleDelete} disabled={loading} className="grow" variant="destructive">Delete {loading ? <Loader2 className="animate-spin" /> : <></>}</Button>
                </div>
            </DialogContent>
        </Dialog>
    )
} 