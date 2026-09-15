import AdminDashboard from "#components/admin/Dashboard";
import AdminLayout from "#components/admin/Layout";
import AdminSettings from "#components/admin/Settings";
import AdminUsers from "#components/admin/Users";
import AdminVouchRates from "#components/admin/VouchRates";
import { Button } from "#components/ui/button";
import { Input } from "#components/ui/input";
import { Label } from "#components/ui/label";
import { toast } from "#components/ui/toast";
import { encrypt } from "#lib/utils";
import { ArrowRight, Loader2 } from "lucide-react";
import { useEffect, useState } from "react";
import { Route, Switch, useLocation } from "wouter";

function AdminLogin() {
    const [key, setKey] = useState("");
    const [isLoadling, setIsLoading] = useState(false);
    const [, setLocation] = useLocation();

    async function login(k: string, p: string) {
        setIsLoading(true)
        const res = await fetch("http://localhost:8000/api/admin/dashboard?payload=" + await encrypt(k, p));
        if (res.ok) {
            window.localStorage.setItem("adminkey", k);
            setLocation("/dashboard");
        } else {
            toast.error("Invalid admin key", {
                position: "top-center"
            });
        }
        setIsLoading(false)
    }

    useEffect(() => {
        (async () => {
            const k = window.localStorage.getItem("adminkey");
            if (k) await login(k, JSON.stringify({ "id": 1 }))
        })()
    }, []);

    return (
        <>
            <div className="h-screen [@supports(height:100dvh)]:h-dvh w-screen grid place-items-center p-4">
                <div className="max-w-sm w-full">
                    <Label htmlFor="adminkey" className="text-md font-bold flex justify-center w-full">Admin Key</Label>
                    <p className="text-center text-sm text-foreground/60 pb-4">
                        Enter your admin key to proceed.
                    </p>
                    <Input className="text-center" onChange={(e) => setKey(e.target.value)} value={key} id="adminkey" placeholder="e.g skfujrkehjdysi" />

                    <div className="flex justify-center pt-6">
                        <Button disabled={isLoadling || !key} onClick={async () => await login(key, JSON.stringify({ "id": 1 }))} className="cursor-pointer" size="icon-lg">
                            {isLoadling ? <Loader2 className="animate-spin" /> : <ArrowRight />}
                        </Button>
                    </div>
                </div>
            </div>
        </>
    )
}

export function Admin() {
    return (
        <Switch>
            <Route path="/"><AdminLogin /></Route>
            <AdminLayout>
                <Route path="/dashboard"><AdminDashboard /></Route>
                <Route path="/users"><AdminUsers /></Route>
                <Route path="/vouch_rates"><AdminVouchRates /></Route>
                <Route path="/settings"><AdminSettings /></Route>
            </AdminLayout>
        </Switch>
    )
}

export default Admin;