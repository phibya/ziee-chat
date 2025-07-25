import { create } from "zustand";

interface AddLocalModelUploadDrawerState {
  open: boolean;
  loading: boolean;
  providerId: string | null;
}

export const useAddLocalModelUploadDrawerStore =
  create<AddLocalModelUploadDrawerState>(() => ({
    open: false,
    loading: false,
    providerId: null,
  }));

// Modal actions
export const openAddLocalModelUploadDrawer = (providerId: string) => {
  useAddLocalModelUploadDrawerStore.setState({
    open: true,
    providerId,
  });
};

export const closeAddLocalModelUploadDrawer = () => {
  useAddLocalModelUploadDrawerStore.setState({
    open: false,
    loading: false,
    providerId: null,
  });
};

export const setAddLocalModelUploadDrawerLoading = (loading: boolean) => {
  useAddLocalModelUploadDrawerStore.setState({
    loading,
  });
};
