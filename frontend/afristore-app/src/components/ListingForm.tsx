// ─────────────────────────────────────────────────────────────
// components/ListingForm.tsx — create and edit listing form
// ─────────────────────────────────────────────────────────────

"use client";

import { useState, useRef, useEffect } from "react";
import Image from "next/image";
import { useCreateListing, useUpdateListing, UpdateListingInput, CreateListingInput } from "@/hooks/useMarketplace";
import { useWalletContext } from "@/context/WalletContext";
import { Upload, CheckCircle, Loader2, Save } from "lucide-react";
import { GuardButton } from "./WalletGuard";
import { ArtworkMetadata, fetchMetadata, cidToGatewayUrl } from "@/lib/ipfs";
import { Listing, stroopsToXlm } from "@/lib/contract";

interface ListingFormProps {
  listing?: Listing; // If provided, we are in EDIT mode
  onSuccess?: (listingId: number) => void;
  onCancel?: () => void;
}

export function ListingForm({ listing, onSuccess, onCancel }: ListingFormProps) {
  const isEdit = !!listing;
  const { publicKey } = useWalletContext();
  
  const { create, isCreating, progress: createProgress, error: createError } = useCreateListing(publicKey);
  const { update, isUpdating, progress: updateProgress, error: updateError } = useUpdateListing(publicKey);

  const [preview, setPreview] = useState<string | null>(null);
  const [form, setForm] = useState({
    title: "",
    description: "",
    artistName: "",
    year: new Date().getFullYear().toString(),
    priceXlm: 10,
    tokenAddress: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC", // XLM Native
  });
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [successId, setSuccessId] = useState<number | null>(null);
  const [currentMetadata, setCurrentMetadata] = useState<ArtworkMetadata | null>(null);
  const [isFetchingMetadata, setIsFetchingMetadata] = useState(false);
  
  const fileRef = useRef<HTMLInputElement>(null);

  // Load existing data if in edit mode
  useEffect(() => {
    if (listing) {
      setIsFetchingMetadata(true);
      fetchMetadata(listing.metadata_cid)
        .then((meta) => {
          setCurrentMetadata(meta);
          setForm({
            title: meta.title,
            description: meta.description,
            artistName: meta.artist,
            year: meta.year,
            priceXlm: parseFloat(stroopsToXlm(listing.price)),
            tokenAddress: listing.token,
          });
          setPreview(cidToGatewayUrl(meta.image));
        })
        .finally(() => setIsFetchingMetadata(false));
    }
  }, [listing]);

  const handleFile = (file: File) => {
    setSelectedFile(file);
    const url = URL.createObjectURL(file);
    setPreview(url);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer.files[0];
    if (file && file.type.startsWith("image/")) handleFile(file);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (isEdit && listing && currentMetadata) {
      const success = await update({
        listingId: listing.listing_id,
        ...form,
        imageFile: selectedFile || undefined,
        currentMetadata,
      });
      if (success) {
        setSuccessId(listing.listing_id);
        onSuccess?.(listing.listing_id);
      }
    } else if (!isEdit) {
      if (!selectedFile) return;
      const id = await create({ ...form, imageFile: selectedFile });
      if (id !== null) {
        setSuccessId(id);
        onSuccess?.(id);
      }
    }
  };

  const isLoading = isCreating || isUpdating || isFetchingMetadata;
  const progress = isEdit ? updateProgress : createProgress;
  const error = isEdit ? updateError : createError;

  if (successId !== null) {
    return (
      <div className="flex flex-col items-center gap-4 rounded-2xl border border-green-200 bg-green-50 p-8 text-center">
        <CheckCircle size={48} className="text-green-500" />
        <h3 className="text-xl font-semibold text-green-800">
          Listing #{successId} {isEdit ? "updated" : "created"}!
        </h3>
        <p className="text-sm text-green-700">
          Your changes are now live on the Afristore marketplace.
        </p>
        <div className="flex gap-3 mt-4">
            {!isEdit && (
                <button
                onClick={() => {
                    setSuccessId(null);
                    setPreview(null);
                    setSelectedFile(null);
                    setForm({ title: "", description: "", artistName: "", year: new Date().getFullYear().toString(), priceXlm: 10, tokenAddress: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC" });
                }}
                className="rounded-lg bg-green-600 px-5 py-2 text-sm font-medium text-white hover:bg-green-700"
                >
                List Another
                </button>
            )}
            <button
                onClick={onCancel}
                className="rounded-lg border border-green-200 bg-white px-5 py-2 text-sm font-medium text-green-700 hover:bg-green-100"
            >
                Back to Dashboard
            </button>
        </div>
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Image upload zone */}
      <div
        onDrop={handleDrop}
        onDragOver={(e) => e.preventDefault()}
        onClick={() => !isEdit && fileRef.current?.click()}
        className={`relative flex flex-col items-center justify-center rounded-2xl border-2 border-dashed border-brand-300 bg-brand-50 p-8 text-center transition-colors ${!isEdit ? "cursor-pointer hover:border-brand-500" : ""}`}
      >
        {preview ? (
          <div className="relative h-48 w-full">
            <Image src={preview} alt="Preview" fill className="object-contain rounded-xl" unoptimized />
            {!isEdit && (
                <div className="absolute inset-0 bg-black/40 opacity-0 hover:opacity-100 flex items-center justify-center rounded-xl transition-opacity">
                    <p className="text-white text-sm font-medium">Click to change</p>
                </div>
            )}
          </div>
        ) : (
          <>
            <Upload size={36} className="mb-2 text-brand-400" />
            <p className="text-sm font-medium text-brand-600">
              Drop artwork here or click to browse
            </p>
            <p className="mt-1 text-xs text-gray-400">PNG, JPG, GIF, WEBP — max 50 MB</p>
          </>
        )}
        <input
          ref={fileRef}
          type="file"
          accept="image/*"
          className="hidden"
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) handleFile(file);
          }}
        />
      </div>

      {/* Fields */}
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="sm:col-span-2">
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Title *
          </label>
          <input
            required
            value={form.title}
            onChange={(e) => setForm({ ...form, title: e.target.value })}
            className="w-full rounded-lg border border-gray-200 px-3 py-2 text-sm focus:border-brand-500 focus:outline-none"
            placeholder="e.g. Sunlit Savanna"
          />
        </div>

        <div className="sm:col-span-2">
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Description
          </label>
          <textarea
            rows={3}
            value={form.description}
            onChange={(e) => setForm({ ...form, description: e.target.value })}
            className="w-full rounded-lg border border-gray-200 px-3 py-2 text-sm focus:border-brand-500 focus:outline-none"
            placeholder="Tell the story behind this piece…"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Artist Name *
          </label>
          <input
            required
            value={form.artistName}
            onChange={(e) => setForm({ ...form, artistName: e.target.value })}
            className="w-full rounded-lg border border-gray-200 px-3 py-2 text-sm focus:border-brand-500 focus:outline-none"
            placeholder="Your name or alias"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Year *
          </label>
          <input
            required
            type="number"
            min={1900}
            max={2100}
            value={form.year}
            onChange={(e) => setForm({ ...form, year: e.target.value })}
            className="w-full rounded-lg border border-gray-200 px-3 py-2 text-sm focus:border-brand-500 focus:outline-none"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Price (XLM) *
          </label>
          <div className="relative">
            <input
              required
              type="number"
              min={0.0000001}
              step="any"
              value={form.priceXlm}
              onChange={(e) => setForm({ ...form, priceXlm: parseFloat(e.target.value) })}
              className="w-full rounded-lg border border-gray-200 px-3 py-2 pr-14 text-sm focus:border-brand-500 focus:outline-none"
            />
            <span className="absolute right-3 top-1/2 -translate-y-1/2 text-sm font-medium text-gray-400">
              XLM
            </span>
          </div>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Payment Token *
          </label>
          <input
            required
            value={form.tokenAddress}
            onChange={(e) => setForm({ ...form, tokenAddress: e.target.value })}
            className="w-full rounded-lg border border-gray-200 px-3 py-2 text-sm font-mono focus:border-brand-500 focus:outline-none"
            placeholder="Contract ID (Native XLM by default)"
          />
        </div>
      </div>

      {/* Progress / error */}
      {isLoading && progress && (
        <div className="flex items-center gap-2 rounded-lg bg-brand-50 px-4 py-3 text-sm text-brand-700">
          <Loader2 size={16} className="animate-spin" />
          {progress}
        </div>
      )}
      {error && (
        <p className="rounded-lg bg-red-50 px-4 py-3 text-sm text-red-600">
          {error}
        </p>
      )}

      {/* Buttons */}
      <div className="flex gap-4">
        {isEdit && (
            <button
                type="button"
                onClick={onCancel}
                disabled={isLoading}
                className="flex-1 rounded-xl border border-gray-200 py-3.5 text-base font-medium text-gray-600 hover:bg-gray-50 transition-all disabled:opacity-50"
            >
                Cancel
            </button>
        )}
        <GuardButton
            type="submit"
            disabled={isLoading || (!isEdit && !selectedFile)}
            actionName={isEdit ? "To update your listing" : "To list your artwork"}
            className="flex-[2] flex items-center justify-center gap-2 rounded-xl bg-brand-500 py-3.5 text-base font-bold text-white shadow-xl shadow-brand-500/20 hover:bg-brand-600 transition-all active:scale-[0.98] disabled:opacity-50"
        >
            {isLoading ? (
                <>
                    <Loader2 size={20} className="animate-spin" />
                    {progress || "Processing…"}
                </>
            ) : (
                <>
                    {isEdit ? <Save size={20} /> : <Upload size={20} />}
                    {isEdit ? "Update Listing" : "List Artwork"}
                </>
            )}
        </GuardButton>
      </div>
    </form>
  );
}
