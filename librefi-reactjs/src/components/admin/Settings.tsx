import { useEffect, useState } from "react";
import { encrypt, getAdminKey } from "#lib/utils";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { toast } from "../ui/toast";
import { Loader2 } from "lucide-react";
import { Switch } from "../ui/switch";
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "../ui/dialog";

export default function AdminSettings() {
    const [qos, setQos] = useState({ download: 0, upload: 0, global_download: 0, global_upload: 0 });
    const [wifi, setWifi] = useState({
        ssid_2g: "", key_2g: "", disabled_2g: false, supports_2g: true,
        ssid_5g: "", key_5g: "", disabled_5g: false, supports_5g: false
    });
    const [coinslot, setCoinslot] = useState({ prefix: "", generated_at: "" });
    const [newCoinslotKey, setNewCoinslotKey] = useState("");

    const [loadingQos, setLoadingQos] = useState(false);
    const [loadingWifi, setLoadingWifi] = useState(false);
    const [loadingCoinslot, setLoadingCoinslot] = useState(false);

    async function loadSettings() {
        const key = getAdminKey();
        const payload = await encrypt(key, JSON.stringify({ action: "get" }));

        const resQos = await fetch("/api/admin/qos");
        if (resQos.ok) {
            const data = await resQos.json();
            setQos({
                download: data.download || 0,
                upload: data.upload || 0,
                global_download: data.global_download || 0,
                global_upload: data.global_upload || 0
            });
        }

        const resWifi = await fetch(`/api/admin/wifi?payload=${payload}`);
        if (resWifi.ok) {
            const data = await resWifi.json();
            setWifi(data);
        }

        const resCoinslot = await fetch(`/api/admin/coinslot-key?payload=${payload}`);
        if (resCoinslot.ok) {
            const data = await resCoinslot.json();
            setCoinslot(data);
        }
    }

    useEffect(() => {
        loadSettings();
    }, []);

    async function saveQos() {
        setLoadingQos(true);
        const res = await fetch("/api/admin/qos", {
            method: "POST",
            body: JSON.stringify({ payload: await encrypt(getAdminKey(), JSON.stringify(qos)) })
        });
        if (res.ok) {
            toast.success("Success", { position: "top-center", description: "QoS settings applied successfully." });
        } else {
            toast.error("Error", { position: "top-center", description: "Failed to apply QoS settings." });
        }
        setLoadingQos(false);
    }

    async function saveWifi() {
        setLoadingWifi(true);
        const res = await fetch("/api/admin/wifi", {
            method: "POST",
            body: JSON.stringify({ payload: await encrypt(getAdminKey(), JSON.stringify(wifi)) })
        });
        if (res.ok) {
            toast.success("Success", { position: "top-center", description: "WiFi configuration updated successfully." });
        } else {
            toast.error("Error", { position: "top-center", description: "Failed to update WiFi configuration." });
        }
        setLoadingWifi(false);
    }

    async function generateCoinslotKey() {
        setLoadingCoinslot(true);
        const res = await fetch("/api/admin/coinslot-key", {
            method: "POST",
            body: JSON.stringify({ payload: await encrypt(getAdminKey(), JSON.stringify({ action: "generate" })) })
        });
        if (res.ok) {
            const data = await res.json();
            setNewCoinslotKey(data.key);
            toast.success("Success", { position: "top-center", description: "New Coinslot Key generated." });
            loadSettings();
        } else {
            toast.error("Error", { position: "top-center", description: "Failed to generate Coinslot Key." });
        }
        setLoadingCoinslot(false);
    }

    return (
        <div className="p-4 pt-0 flex flex-col gap-6 pb-20 mx-auto">
            <div className="border border-border p-6 rounded-xl flex flex-col gap-4">
                <div>
                    <h2 className="text-lg font-semibold">Quality of Service (QoS)</h2>
                    <p className="text-sm text-foreground/60">Set bandwidth limits per client. Set to 0 to disable.</p>
                </div>
                <div className="flex flex-col gap-4">
                    <div className="flex flex-col md:flex-row gap-4">
                        <div className="flex-1 space-y-1.5">
                            <Label>Per-User Download (Mbps)</Label>
                            <Input type="number" step="0.1" value={qos.download || ""} placeholder="0" onChange={(e) => setQos({ ...qos, download: parseFloat(e.target.value) || 0 })} />
                        </div>
                        <div className="flex-1 space-y-1.5">
                            <Label>Per-User Upload (Mbps)</Label>
                            <Input type="number" step="0.1" value={qos.upload || ""} placeholder="0" onChange={(e) => setQos({ ...qos, upload: parseFloat(e.target.value) || 0 })} />
                        </div>
                    </div>
                    <div className="flex flex-col md:flex-row gap-4">
                        <div className="flex-1 space-y-1.5">
                            <Label>Global Download Cap (Mbps)</Label>
                            <Input type="number" step="0.1" value={qos.global_download || ""} placeholder="0" onChange={(e) => setQos({ ...qos, global_download: parseFloat(e.target.value) || 0 })} />
                        </div>
                        <div className="flex-1 space-y-1.5">
                            <Label>Global Upload Cap (Mbps)</Label>
                            <Input type="number" step="0.1" value={qos.global_upload || ""} placeholder="0" onChange={(e) => setQos({ ...qos, global_upload: parseFloat(e.target.value) || 0 })} />
                        </div>
                    </div>
                </div>
                <div className="flex justify-end">
                    <Dialog>
                        <DialogTrigger render={
                            <Button disabled={loadingQos}>
                                {loadingQos ? <Loader2 className="animate-spin mr-2" size={16} /> : null}
                                Apply QoS
                            </Button>}></DialogTrigger>
                        <DialogContent>
                            <DialogHeader>
                                <DialogTitle>Apply QoS Settings?</DialogTitle>
                                <DialogDescription>
                                    This will immediately affect network bandwidth for connected users.
                                </DialogDescription>
                            </DialogHeader>
                            <DialogFooter>
                                <DialogClose render={<Button variant="outline">Cancel</Button>}></DialogClose>
                                <DialogClose render={<Button onClick={saveQos}>Apply</Button>}></DialogClose>
                            </DialogFooter>
                        </DialogContent>
                    </Dialog>
                </div>
            </div>

            <div className="border border-border p-6 rounded-xl flex flex-col gap-6 relative">
                <div>
                    <h2 className="text-lg font-semibold">WiFi Configuration</h2>
                    <p className="text-sm text-foreground/60">Configure your 2.4GHz and 5GHz access points. Leave password blank for an open network.</p>
                </div>

                {!wifi.supports_2g && !wifi.supports_5g ? (
                    <div className="bg-foreground/2 p-6 rounded-md grid place-items-center text-foreground/70 absolute h-full w-full top-0 left-0 backdrop-blur-lg">
                        <p>No WiFi interfaces detected on this device.</p>
                    </div>
                ) : (
                    <>
                        {wifi.supports_2g && (
                            <div className="flex flex-col gap-4 border-l-4 border-primary pl-4">
                                <h3 className="font-semibold text-md flex items-center justify-between">
                                    2.4GHz Network
                                    <div className="flex items-center gap-2">
                                        <Label className="text-xs">Enabled</Label>
                                        <Switch checked={!wifi.disabled_2g} onCheckedChange={(c) => setWifi({ ...wifi, disabled_2g: !c })} />
                                    </div>
                                </h3>
                                <div className="flex flex-col md:flex-row gap-4">
                                    <div className="flex-1 space-y-1.5">
                                        <Label>SSID (Name)</Label>
                                        <Input value={wifi.ssid_2g} onChange={(e) => setWifi({ ...wifi, ssid_2g: e.target.value })} disabled={wifi.disabled_2g} />
                                    </div>
                                    <div className="flex-1 space-y-1.5">
                                        <Label>Password (Key)</Label>
                                        <Input type="password" value={wifi.key_2g} onChange={(e) => setWifi({ ...wifi, key_2g: e.target.value })} disabled={wifi.disabled_2g} placeholder="Leave blank for none" />
                                    </div>
                                </div>
                            </div>
                        )}

                        {wifi.supports_5g && (
                            <div className="flex flex-col gap-4 border-l-4 border-primary pl-4">
                                <h3 className="font-semibold text-md flex items-center justify-between">
                                    5GHz Network
                                    <div className="flex items-center gap-2">
                                        <Label className="text-xs">Enabled</Label>
                                        <Switch checked={!wifi.disabled_5g} onCheckedChange={(c) => setWifi({ ...wifi, disabled_5g: !c })} />
                                    </div>
                                </h3>
                                <div className="flex flex-col md:flex-row gap-4">
                                    <div className="flex-1 space-y-1.5">
                                        <Label>SSID (Name)</Label>
                                        <Input value={wifi.ssid_5g} onChange={(e) => setWifi({ ...wifi, ssid_5g: e.target.value })} disabled={wifi.disabled_5g} />
                                    </div>
                                    <div className="flex-1 space-y-1.5">
                                        <Label>Password (Key)</Label>
                                        <Input type="password" value={wifi.key_5g} onChange={(e) => setWifi({ ...wifi, key_5g: e.target.value })} disabled={wifi.disabled_5g} placeholder="Leave blank for none" />
                                    </div>
                                </div>
                            </div>
                        )}

                        <div className="flex justify-end">
                            <Dialog>
                                <DialogTrigger render={
                                    <Button disabled={loadingWifi}>
                                        {loadingWifi ? <Loader2 className="animate-spin mr-2" size={16} /> : null}
                                        Save WiFi Settings
                                    </Button>
                                }>
                                </DialogTrigger>
                                <DialogContent>
                                    <DialogHeader>
                                        <DialogTitle>Save WiFi Settings?</DialogTitle>
                                        <DialogDescription>
                                            Are you sure you want to save these WiFi settings? The access point will restart and temporarily disconnect all users.
                                        </DialogDescription>
                                    </DialogHeader>
                                    <DialogFooter>
                                        <DialogClose render={<Button variant="outline">Cancel</Button>}> </DialogClose>
                                        <DialogClose render={<Button onClick={saveWifi}>Save</Button>}></DialogClose>
                                    </DialogFooter>
                                </DialogContent>
                            </Dialog>
                        </div>
                    </>
                )}
            </div>

            <div className="border border-border p-6 rounded-xl flex flex-col gap-4">
                <div>
                    <h2 className="text-lg font-semibold text-amber-500">Coinslot Authentication Key</h2>
                    <p className="text-sm text-foreground/60">Generate a new cryptographic key for your ESP8266 Coinslot module. This will invalidate the old key.</p>
                </div>

                <div className="bg-foreground/2 p-4 rounded-md">
                    <p className="text-sm font-medium">Current Key Prefix: <span className="font-mono text-primary">{coinslot.prefix || "None"}</span></p>
                    {coinslot.generated_at && <p className="text-xs text-foreground/60 mt-1">Generated at: {new Date(parseInt(coinslot.generated_at) * 1000).toLocaleString()}</p>}
                </div>

                {newCoinslotKey && (
                    <div className="bg-emerald-100/10 border border-emerald-300/20 p-4 rounded-md">
                        <p className="text-sm font-semibold text-emerald-500 mb-2">New Key Generated (Save this, it won't be shown again):</p>
                        <p className="font-mono text-lg text-emerald-600 break-all select-all">{newCoinslotKey}</p>
                    </div>
                )}

                <div className="flex justify-end mt-2">
                    <Dialog>
                        <DialogTrigger render={<Button disabled={loadingCoinslot} variant="destructive">
                            {loadingCoinslot ? <Loader2 className="animate-spin mr-2" size={16} /> : null}
                            Generate New Key
                        </Button>}></DialogTrigger>
                        <DialogContent>
                            <DialogHeader>
                                <DialogTitle>Generate New Coinslot Key?</DialogTitle>
                                <DialogDescription>
                                    This action will permanently invalidate your current coinslot module key, and it will stop working until you reset it and apply the new key.
                                </DialogDescription>
                            </DialogHeader>
                            <DialogFooter>
                                <DialogClose render={<Button variant="outline">Cancel</Button>}></DialogClose>
                                <DialogClose render={<Button variant="destructive" onClick={generateCoinslotKey}>Generate</Button>}></DialogClose>
                            </DialogFooter>
                        </DialogContent>
                    </Dialog>
                </div>
            </div>
        </div>
    );
}