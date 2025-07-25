import { create } from "zustand";

interface AddLocalModelDownloadDrawerState {
  open: boolean;
  loading: boolean;
  providerId: string | null;
}

export const useAddLocalModelDownloadDrawerStore =
  create<AddLocalModelDownloadDrawerState>(() => ({
    open: false,
    loading: false,
    providerId: null,
  }));

// Modal actions
export const openAddLocalModelDownloadDrawer = (providerId: string) => {
  useAddLocalModelDownloadDrawerStore.setState({
    open: true,
    providerId,
  });
};

export const closeAddLocalModelDownloadDrawer = () => {
  useAddLocalModelDownloadDrawerStore.setState({
    open: false,
    loading: false,
    providerId: null,
  });
};

export const setAddLocalModelDownloadDrawerLoading = (loading: boolean) => {
  useAddLocalModelDownloadDrawerStore.setState({
    loading,
  });
};
