import { encrypt, formatSeconds, getAdminKey } from "#lib/utils";
import { Pencil } from "lucide-react";
import {
    Table,
    TableBody,
    TableCaption,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "../ui/table"
import { Button } from "../ui/button";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "#components/ui/popover"
import { SettingsIcon } from "lucide-react"
import { useLocation } from "wouter";
import { useEffect, useState } from "react";
import { Label } from "../ui/label";
import { Input } from "../ui/input";
import {
    Combobox,
    ComboboxContent,
    ComboboxEmpty,
    ComboboxInput,
    ComboboxItem,
    ComboboxList,
} from "../ui/combobox"
import { toast } from "../ui/toast";

interface UsersT {
    "mac": string
    "name": string
    "ip": string
    "paused": boolean
    "pause_attempts": number
    "pause_day": number
    "paused_on": number
    "expires_on": number
    "new_expiry": number | undefined
}

function AdminUsers() {
    const [_, setLocation] = useLocation();
    const [users, setUsers] = useState<UsersT[] | null>()
    const [forEdit, setForEdit] = useState<UsersT | null>()
    const [page, setPage] = useState(1);
    const [totalPages, setTotalPages] = useState(1);

    async function getUsers() {
        const res = await fetch("http://localhost:8000/api/admin/users?payload=" + await encrypt(getAdminKey(), JSON.stringify({ "id": 1, page })));
        if (res.ok) {
            const json = await res.json();
            setUsers(json.users);
            setTotalPages(json.total_pages);
        } else {
            setLocation("/");
        }
    }

    async function handleEdit() {
        async function post() {
            const res = await fetch("http://localhost:8000/api/admin/users", {
                method: "POST",
                body: JSON.stringify({ payload: await encrypt(getAdminKey(), JSON.stringify(forEdit)) })
            })
            if (res.ok) {
                await getUsers();
            } else {
                throw Error;
            }
        }

        toast.promise(
            post(),
            {
                loading: {
                    title: "Updating user...",
                    description: "Please wait while we update your connection."
                },
                success: {
                    title: "Success",
                    description: "User has been updated successfully."
                },
                error: (err) => ({
                    title: "Update failed",
                    description: err instanceof Error ? err.message : "Something went wrong."
                })
            },
            { position: "top-center" }
        );
    }

    useEffect(() => {
        getUsers();
    }, [page]);

    return (
        <div>
            <div className="px-2">
                <Table>
                    {users?.length !== 0 ? <></> : <TableCaption>No users found.</TableCaption>}
                    <TableHeader>
                        <TableRow>
                            <TableHead className="text-xs font-bold">MAC</TableHead>
                            <TableHead>IP</TableHead>
                            <TableHead>Name</TableHead>
                            <TableHead>Paused</TableHead>
                            <TableHead>Pause Attempts</TableHead>
                            <TableHead>Time Left</TableHead>
                            <TableHead className="text-right text-foreground/50">Edit</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {users && users.map(u => (
                            <TableRow key={u.mac}>
                                <TableCell className="text-xs font-bold">
                                    {u.mac}
                                </TableCell>
                                <TableCell className="text-sm">{u.ip}</TableCell>
                                <TableCell className="text-sm">{u.name.trim() == "" ? "_" : u.name}</TableCell>
                                <TableCell className="text-sm">{String(u.paused)}</TableCell>
                                <TableCell className="text-sm">{u.pause_attempts}</TableCell>
                                <TableCell className="text-sm">{formatSeconds(Math.max(0, u.paused ? u.expires_on - u.paused_on : u.expires_on - Math.floor(Date.now() / 1000)))}</TableCell>
                                <TableCell className="text-sm">
                                    <Popover onOpenChange={(v) => {
                                        if (v) {
                                            setForEdit({ ...u, new_expiry: Math.max(0, u.paused ? u.expires_on - u.paused_on : u.expires_on - Math.floor(Date.now() / 1000)) })
                                        } else {
                                            handleEdit()
                                        }
                                    }
                                    }>
                                        <PopoverTrigger render={<Button size="icon-xs" variant="outline" className="border-primary text-foreground/60"><Pencil size={16} /></Button>
                                        }>
                                            <SettingsIcon aria-hidden="true" />
                                        </PopoverTrigger>
                                        <PopoverContent className="w-72 gap-0 p-0" align="end">
                                            <div className="border-b p-3">
                                                <h4 className="m-0 font-semibold">Edit User</h4>
                                                <p className="text-muted-foreground">View and update user details.</p>
                                            </div>
                                            <div className="space-y-3 p-3 pb-4">
                                                <div className="space-y-1.5">
                                                    <Label>Name</Label>
                                                    <Input placeholder="E.g Junjun" onChange={(e) => forEdit && setForEdit({ ...forEdit, name: e.target.value })} value={forEdit?.name} />
                                                </div>
                                                <div className="space-y-1.5">
                                                    <Label>Paused</Label>
                                                    <Combobox value={forEdit?.paused ? "true" : "false"} onInputValueChange={(v) => forEdit && setForEdit({ ...forEdit, paused: v === "true" })} items={['true', 'false']}>
                                                        <ComboboxInput placeholder="Select" />
                                                        <ComboboxContent>
                                                            <ComboboxEmpty>No items found.</ComboboxEmpty>
                                                            <ComboboxList>
                                                                {(item) => (
                                                                    <ComboboxItem key={item} value={item}>
                                                                        {item}
                                                                    </ComboboxItem>
                                                                )}
                                                            </ComboboxList>
                                                        </ComboboxContent>
                                                    </Combobox>
                                                </div>
                                                <div className="space-y-1.5">
                                                    <Label>Paused Attempts</Label>
                                                    <Input type="number" placeholder="E.g 1" onChange={(e) => forEdit && setForEdit({ ...forEdit, pause_attempts: parseInt(e.target.value) })} value={forEdit?.pause_attempts} />
                                                </div>
                                                <div className="space-y-1.5">
                                                    <Label>Time Left<span className="text-foreground/60">(seconds)</span></Label>
                                                    <Input type="number" placeholder="E.g 900" onChange={(e) => forEdit && setForEdit({ ...forEdit, new_expiry: parseInt(e.target.value) })} value={forEdit?.new_expiry} />
                                                    <p className="text-xs">{formatSeconds(forEdit?.new_expiry || 0)}</p>

                                                </div>
                                            </div>
                                        </PopoverContent>
                                    </Popover>
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>

                <div className="w-full flex justify-center">
                    <div className="flex items-center justify-between mt-4 px-2">
                        <Button
                            variant="outline"
                            size="sm"
                            disabled={page <= 1}
                            onClick={() => setPage(p => Math.max(1, p - 1))}
                        >
                            Prev
                        </Button>
                        <span className="text-sm text-muted-foreground font-medium">
                            Page {page} of {totalPages}
                        </span>
                        <Button
                            variant="outline"
                            size="sm"
                            disabled={page >= totalPages}
                            onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                        >
                            Next
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    )
}

export default AdminUsers;